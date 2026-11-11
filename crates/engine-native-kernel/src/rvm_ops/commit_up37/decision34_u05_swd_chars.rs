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
            let Some(t) = hostile_target(ctx, owner) else {
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
            let Some(t) = hostile_target(ctx, owner) else {
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
                if let Some(t) = hostile_target(ctx, owner) {
                    ctx.start_draw(t, 1);
                }
            }
            set_flag(ctx.core, owner, "u05_buff:cyxianjiang", 1);
        }
        // character/swd/skill.js:105-244: usable 1, discard 1 card -> thunder damage / recover + draw 1
        "gxianyin" =>  { u05_mark_invoked(ctx, owner, "gxianyin"); if !u05_check_usable(ctx, owner, "gxianyin", 1) { return true; } if !u05_pay_discard(ctx, owner, "gxianyin", 1, true) { return true; } u05_recover(ctx, owner, 1); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:245-260
        "yeying" =>  { u05_mark_invoked(ctx, owner, "yeying"); if !u05_check_usable(ctx, owner, "yeying", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "qiankunbiao"), 1); },
        // character/swd/skill.js:261-340
        "juxi" =>  { u05_mark_invoked(ctx, owner, "juxi"); if !u05_check_usable(ctx, owner, "juxi", 1) { return true; } u05_recover(ctx, owner, 1); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:341-397
        "jiefen" =>  { u05_mark_invoked(ctx, owner, "jiefen"); if !u05_check_usable(ctx, owner, "jiefen", 1) { return true; } if !u05_pay_discard(ctx, owner, "jiefen", 1, true) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/swd/skill.js:398-418
        "datong" =>  { u05_mark_invoked(ctx, owner, "datong"); ctx.start_draw(owner, 2); },
        // character/swd/skill.js:419-472
        "huodan" =>  { u05_mark_invoked(ctx, owner, "huodan"); if !u05_check_usable(ctx, owner, "huodan", 1) { return true; } if !u05_pay_discard(ctx, owner, "huodan", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Fire); u05_lose_hp(ctx, owner, 1); },
        // character/swd/skill.js:597-671
        "zhanxing" =>  { u05_mark_invoked(ctx, owner, "zhanxing"); if !u05_check_usable(ctx, owner, "zhanxing", 1) { return true; } if !u05_pay_discard(ctx, owner, "zhanxing", 1, true) { return true; } u05_change_hujia(ctx, owner, 1); u05_recover(ctx, owner, 1); u05_damage_aoe(ctx, owner, 1, DamageNature::Thunder); ctx.start_draw(owner, 2); },
        // character/swd/skill.js:683-719
        "kbolan2" =>  { u05_mark_invoked(ctx, owner, "kbolan2"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:720-741
        "hujing" =>  { u05_mark_invoked(ctx, owner, "hujing"); ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:hujing", 1); },
        // character/swd/skill.js:812-846
        "lingshi" =>  { u05_mark_invoked(ctx, owner, "lingshi"); set_flag(ctx.core, owner, "u05_mod:lingshi", 1); },
        // character/swd/skill.js:847-877
        "tiebi" =>  { u05_mark_invoked(ctx, owner, "tiebi"); u05_change_hujia(ctx, owner, 1); },
        // character/swd/skill.js:878-929
        "shenyan" =>  { u05_mark_invoked(ctx, owner, "shenyan"); if !u05_check_limited(ctx, owner, "shenyan") { return true; } let _ = u05_store_pile(ctx, owner, "u05_pile:shenyan"); },
        // character/swd/skill.js:930-946
        "shenyan2" =>  { u05_mark_invoked(ctx, owner, "shenyan2"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/swd/skill.js:947-969
        "senluo" =>  { u05_mark_invoked(ctx, owner, "senluo"); if !u05_check_usable(ctx, owner, "senluo", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:970-1112
        "xuanying" =>  { u05_mark_invoked(ctx, owner, "xuanying"); if let Some(t) = hostile_target(ctx, owner) { u05_toggle_link(ctx, t); } },
        // character/swd/skill.js:1113-1148
        "hwendao" =>  { u05_mark_invoked(ctx, owner, "hwendao"); let _ = u05_prey_other(ctx, owner, true); },
        // character/swd/skill.js:1149-1185
        "lingfeng" =>  { u05_mark_invoked(ctx, owner, "lingfeng"); u05_change_hujia(ctx, owner, 1); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:1393-1432
        "hlingbo" =>  { u05_mark_invoked(ctx, owner, "hlingbo"); ctx.start_draw(owner, 2); },
        // character/swd/skill.js:1433-1479
        "gtiandao" =>  { u05_mark_invoked(ctx, owner, "gtiandao"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:gtiandao", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/swd/skill.js:1480-1542
        "jinlin" =>  { u05_mark_invoked(ctx, owner, "jinlin"); if !u05_check_limited(ctx, owner, "jinlin") { return true; } u05_change_hujia(ctx, owner, 3); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:jinlin", 1); } },
        // character/swd/skill.js:1543-1559
        "jinlin2" =>  { u05_mark_invoked(ctx, owner, "jinlin2"); u05_change_hujia(ctx, owner, -1); },
        // character/swd/skill.js:1560-1572
        "lingyue" =>  { u05_mark_invoked(ctx, owner, "lingyue"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:1573-1590
        "fengze" =>  { u05_mark_invoked(ctx, owner, "fengze"); if !u05_check_usable(ctx, owner, "fengze", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "taoyuan"), 1); },
        // character/swd/skill.js:1591-1664
        "huanxia" =>  { u05_mark_invoked(ctx, owner, "huanxia"); let _ = u05_store_pile(ctx, owner, "u05_pile:huanxia"); },
        // character/swd/skill.js:1665-1675
        "huajing" =>  { u05_mark_invoked(ctx, owner, "huajing"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:1703-1806
        "bingfeng" =>  { u05_mark_invoked(ctx, owner, "bingfeng"); if !u05_check_limited(ctx, owner, "bingfeng") { return true; } u05_change_max_hp(ctx, owner, -(1)); },
        // character/swd/skill.js:1829-1891
        "yudun" =>  { u05_mark_invoked(ctx, owner, "yudun"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/swd/skill.js:1892-1986
        "guozao" =>  { u05_mark_invoked(ctx, owner, "guozao"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:1987-2030
        "heihuo" =>  { u05_mark_invoked(ctx, owner, "heihuo"); u05_damage_other(ctx, owner, 3, DamageNature::Fire); },
        // character/swd/skill.js:2045-2060
        "yaotong1" =>  { u05_mark_invoked(ctx, owner, "yaotong1"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/swd/skill.js:2061-2076
        "yaotong2" =>  { u05_mark_invoked(ctx, owner, "yaotong2"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "shan"), 1); },
        // character/swd/skill.js:2077-2099
        "yaotong3" =>  { u05_mark_invoked(ctx, owner, "yaotong3"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "wuxie"), 1); },
        // character/swd/skill.js:2100-2122
        "yaotong4" =>  { u05_mark_invoked(ctx, owner, "yaotong4"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "tao"), 1); },
        // character/swd/skill.js:2123-2156
        "pojian" =>  { u05_mark_invoked(ctx, owner, "pojian"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:2157-2191
        "huajin" =>  { u05_mark_invoked(ctx, owner, "huajin"); if !u05_check_usable(ctx, owner, "huajin", 1) { return true; } if !u05_pay_discard(ctx, owner, "huajin", 1, true) { return true; } set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/swd/skill.js:2208-2242
        "yuchen" =>  { u05_mark_invoked(ctx, owner, "yuchen"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:2243-2276
        "bingjian" =>  { u05_mark_invoked(ctx, owner, "bingjian"); if !u05_check_usable(ctx, owner, "bingjian", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Thunder); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:2385-2450
        "xianghui" =>  { u05_mark_invoked(ctx, owner, "xianghui"); if !u05_check_usable(ctx, owner, "xianghui", 1) { return true; } if !u05_pay_discard(ctx, owner, "xianghui", 1, true) { return true; } u05_recover(ctx, owner, 1); },
        // character/swd/skill.js:2504-2541
        "shangxi" =>  { u05_mark_invoked(ctx, owner, "shangxi"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:2542-2582
        "fuyan" =>  { u05_mark_invoked(ctx, owner, "fuyan"); u05_change_hujia(ctx, owner, 1); },
        // character/swd/skill.js:2611-2619
        "guaili2" => { u05_mark_invoked(ctx, owner, "guaili2"); let _ = u05_discard_up_to(ctx, owner, 2); }
        // character/swd/skill.js:2620-2670
        "xingzhui" =>  { u05_mark_invoked(ctx, owner, "xingzhui"); if !u05_check_usable(ctx, owner, "xingzhui", 1) { return true; } if !u05_pay_discard(ctx, owner, "xingzhui", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:2776-2844
        "shouyin" =>  { u05_mark_invoked(ctx, owner, "shouyin"); if !u05_check_limited(ctx, owner, "shouyin") { return true; } u05_recover(ctx, owner, 1); u05_turn_over(ctx, owner); },
        // character/swd/skill.js:2845-2853
        "sliufeng" =>  { u05_mark_invoked(ctx, owner, "sliufeng"); set_flag(ctx.core, owner, "u05_mod:sliufeng", 1); },
        // character/swd/skill.js:2854-2873
        "linyun" =>  { u05_mark_invoked(ctx, owner, "linyun"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/swd/skill.js:2901-2912
        "linyun3" =>  { u05_mark_invoked(ctx, owner, "linyun3"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:2914-2976
        "hutian" =>  { u05_mark_invoked(ctx, owner, "hutian"); u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:hutian", 1); } },
        // character/swd/skill.js:3133-3145
        "guiying" =>  { u05_mark_invoked(ctx, owner, "guiying"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "toulianghuanzhu"), 1); },
        // character/swd/skill.js:3175-3253
        "ningxian" =>  { u05_mark_invoked(ctx, owner, "ningxian"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:3254-3288
        "xuanyuan" =>  { u05_mark_invoked(ctx, owner, "xuanyuan"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:3308-3363
        "jilve" =>  { u05_mark_invoked(ctx, owner, "jilve"); if !u05_check_usable(ctx, owner, "jilve", 3) { return true; } ctx.start_draw(owner, 1); },
        // character/swd/skill.js:3483-3513
        "duanyue" =>  { u05_mark_invoked(ctx, owner, "duanyue"); if !u05_check_usable(ctx, owner, "duanyue", 1) { return true; } if !u05_pay_discard(ctx, owner, "duanyue", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:3536-3551
        "mojian" =>  { u05_mark_invoked(ctx, owner, "mojian"); u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/swd/skill.js:3552-3561
        "swdliuhong" =>  { u05_mark_invoked(ctx, owner, "swdliuhong"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:3598-3652
        "jianji" =>  { u05_mark_invoked(ctx, owner, "jianji"); if !u05_check_usable(ctx, owner, "jianji", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/swd/skill.js:3653-3674
        "huangyu" =>  { u05_mark_invoked(ctx, owner, "huangyu"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "chiyuxi"), 1); },
        // character/swd/skill.js:3786-3824
        "duishi" =>  { u05_mark_invoked(ctx, owner, "duishi"); if !u05_pay_discard(ctx, owner, "duishi", 1, true) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/swd/skill.js:3875-3898
        "mufeng" =>  { u05_mark_invoked(ctx, owner, "mufeng"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:3924-3962
        "touxi" =>  { u05_mark_invoked(ctx, owner, "touxi"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:3971-4022
        "nlianji" =>  { u05_mark_invoked(ctx, owner, "nlianji"); if !u05_check_usable(ctx, owner, "nlianji", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); if let Some(t) = hostile_target(ctx, owner) { if let (Some(mc), Some(tc)) = (choose_own_card(ctx, owner, false), choose_enemy_card(ctx, owner, t)) { let mr = ctx.catalog.cards.get(mc as usize).map(|c| c.rank).unwrap_or(0); let tr = ctx.catalog.cards.get(tc as usize).map(|c| c.rank).unwrap_or(0); ctx.take_hand_card(owner, mc); super::commit::discard_some(ctx, owner, vec![mc]); ctx.take_hand_card(t, tc); super::commit::discard_some(ctx, t, vec![tc]); let won = mr > tr; set_flag(ctx.core, owner, &format!("u05_pindian_won:{}", "nlianji"), u32::from(won)); if won { ctx.start_draw(owner, 1); } } } },
        // character/swd/skill.js:4034-4046
        "lianji4" =>  { u05_mark_invoked(ctx, owner, "lianji4"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); },
        // character/swd/skill.js:4064-4073
        "miedao2" => {
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as usize;
            if lost > 0 {
                u05_mark_invoked(ctx, owner, "miedao2");
                let _ = u05_discard_up_to(ctx, owner, lost);
            }
        }
        // character/swd/skill.js:4074-4131
        "aojian" =>  { u05_mark_invoked(ctx, owner, "aojian"); if !u05_check_usable(ctx, owner, "aojian", 1) { return true; } if !u05_pay_discard(ctx, owner, "aojian", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:4132-4192
        "moyan" =>  { u05_mark_invoked(ctx, owner, "moyan"); if !u05_check_usable(ctx, owner, "moyan", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/swd/skill.js:4193-4227
        "wangchen" =>  { u05_mark_invoked(ctx, owner, "wangchen"); if let Some(t) = hostile_target(ctx, owner) { u05_turn_over(ctx, t); } },
        // character/swd/skill.js:4294-4319
        "youyin" =>  { u05_mark_invoked(ctx, owner, "youyin"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:4320-4352
        "anlianying" =>  { u05_mark_invoked(ctx, owner, "anlianying"); ctx.start_draw(owner, 2); },
        // character/swd/skill.js:4353-4389
        "zhanlu" =>  { u05_mark_invoked(ctx, owner, "zhanlu"); if !u05_check_usable(ctx, owner, "zhanlu", 1) { return true; } u05_recover(ctx, owner, 1); },
        // character/swd/skill.js:4390-4411
        "huopu" =>  { u05_mark_invoked(ctx, owner, "huopu"); if !u05_check_usable(ctx, owner, "huopu", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "liuxinghuoyu"), 1); },
        // character/swd/skill.js:4412-4426
        "rexue" =>  { u05_mark_invoked(ctx, owner, "rexue"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:4427-4440
        "shengshou" =>  { u05_mark_invoked(ctx, owner, "shengshou"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "caoyao"), 1); },
        // character/swd/skill.js:4441-4463
        "huanjian" =>  { u05_mark_invoked(ctx, owner, "huanjian"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "bingpotong"), 1); },
        // character/swd/skill.js:4464-4474
        "moyu" =>  { u05_mark_invoked(ctx, owner, "moyu"); u05_recover(ctx, owner, 1); },
        // character/swd/skill.js:4535-4633
        "kunlunjing1" =>  { u05_mark_invoked(ctx, owner, "kunlunjing1"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:4634-4675
        "kunlunjing2" =>  { u05_mark_invoked(ctx, owner, "kunlunjing2"); let _ = u05_store_pile(ctx, owner, "u05_pile:kunlunjing2"); },
        // character/swd/skill.js:4676-4722
        "oldliaoyuan" =>  { u05_mark_invoked(ctx, owner, "oldliaoyuan"); if !u05_check_usable(ctx, owner, "oldliaoyuan", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:4768-4855
        "shehun" =>  { u05_mark_invoked(ctx, owner, "shehun"); if !u05_check_usable(ctx, owner, "shehun", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:4936-4981
        "dunxing" =>  { u05_mark_invoked(ctx, owner, "dunxing"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:dunxing", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/swd/skill.js:5159-5185
        "guanhu" =>  { u05_mark_invoked(ctx, owner, "guanhu"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:5196-5239
        "huanhun" =>  { u05_mark_invoked(ctx, owner, "huanhun"); u05_recover(ctx, owner, 1); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:huanhun", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/swd/skill.js:5240-5260
        "yinyue" =>  { u05_mark_invoked(ctx, owner, "yinyue"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:5361-5409
        "lqingcheng" =>  { u05_mark_invoked(ctx, owner, "lqingcheng"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:lqingcheng", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/swd/skill.js:5456-5568
        "shengong" =>  { u05_mark_invoked(ctx, owner, "shengong"); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:5601-5617
        "polang" =>  { u05_mark_invoked(ctx, owner, "polang"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:5660-5704
        "xielei" =>  { u05_mark_invoked(ctx, owner, "xielei"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); },
        // character/swd/skill.js:5705-5765
        "pozhen" =>  { u05_mark_invoked(ctx, owner, "pozhen"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:5766-5802
        "tanlin_defence" =>  { u05_mark_invoked(ctx, owner, "tanlin_defence"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:5860-5874
        "tanlin3" =>  { u05_mark_invoked(ctx, owner, "tanlin3"); set_flag(ctx.core, owner, "u05_mod:tanlin3", 1); },
        // character/swd/skill.js:5875-5883
        "tanlin4" =>  { u05_mark_invoked(ctx, owner, "tanlin4"); set_flag(ctx.core, owner, "u05_mod:tanlin4", 1); },
        // character/swd/skill.js:5884-5924
        "yunchou" =>  { u05_mark_invoked(ctx, owner, "yunchou"); if !u05_check_usable(ctx, owner, "yunchou", 1) { return true; } if !u05_pay_discard(ctx, owner, "yunchou", 1, true) { return true; } ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:5925-5949
        "jqimou" =>  { u05_mark_invoked(ctx, owner, "jqimou"); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:6281-6326
        "xingdian" =>  { u05_mark_invoked(ctx, owner, "xingdian"); if !u05_check_usable(ctx, owner, "xingdian", 1) { return true; } if !u05_pay_discard(ctx, owner, "xingdian", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:6356-6397
        "funiao" =>  { u05_mark_invoked(ctx, owner, "funiao"); if !u05_check_usable(ctx, owner, "funiao", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/swd/skill.js:6504-6562
        "ningshuang" =>  { u05_mark_invoked(ctx, owner, "ningshuang"); ctx.start_draw(owner, 1); u05_lose_hp(ctx, owner, 1); },
        // character/swd/skill.js:6563-6618
        "xielv" =>  { u05_mark_invoked(ctx, owner, "xielv"); u05_recover(ctx, owner, 1); },
        // character/swd/skill.js:6619-6655
        "xiaomoyu" =>  { u05_mark_invoked(ctx, owner, "xiaomoyu"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // character/swd/skill.js:6657-6700
        "tianhuo" =>  { u05_mark_invoked(ctx, owner, "tianhuo"); if !u05_check_usable(ctx, owner, "tianhuo", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Fire); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:6701-6741
        "huanyin" =>  { u05_mark_invoked(ctx, owner, "huanyin"); ctx.start_draw(owner, 1); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:huanyin", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/swd/skill.js:6815-6875
        "tianlun" =>  { u05_mark_invoked(ctx, owner, "tianlun"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:tianlun", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/swd/skill.js:6876-6925
        "lanzhi" =>  { u05_mark_invoked(ctx, owner, "lanzhi"); u05_recover(ctx, owner, 1); },
        // character/swd/skill.js:6926-6956
        "duanyi" =>  { u05_mark_invoked(ctx, owner, "duanyi"); if !u05_check_usable(ctx, owner, "duanyi", 1) { return true; } if !u05_pay_discard(ctx, owner, "duanyi", 2, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:7023-7031
        "miles_xueyi" =>  { u05_mark_invoked(ctx, owner, "miles_xueyi"); u05_lose_hp(ctx, owner, 1); },
        // character/swd/skill.js:7032-7071
        "jifeng" =>  { u05_mark_invoked(ctx, owner, "jifeng"); set_flag(ctx.core, owner, "u05_mod:jifeng", 1); },
        // character/swd/skill.js:7072-7118
        "swd_xiuluo" =>  { u05_mark_invoked(ctx, owner, "swd_xiuluo"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:7119-7159
        "mohua2" =>  { u05_mark_invoked(ctx, owner, "mohua2"); u05_recover(ctx, owner, 2); ctx.start_draw(owner, 2); },
        // character/swd/skill.js:7287-7315
        "duijue2" =>  { u05_mark_invoked(ctx, owner, "duijue2"); set_flag(ctx.core, owner, "u05_mod:duijue2", 1); },
        // character/swd/skill.js:7316-7342
        "yueren" =>  { u05_mark_invoked(ctx, owner, "yueren"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:7344-7366
        "busi" =>  { u05_mark_invoked(ctx, owner, "busi"); u05_recover(ctx, owner, 1); u05_turn_over(ctx, owner); },
        // character/swd/skill.js:7367-7386
        "xuying" =>  { u05_mark_invoked(ctx, owner, "xuying"); u05_lose_hp(ctx, owner, 1); },
        // character/swd/skill.js:7490-7499
        "guiyan2" =>  { u05_mark_invoked(ctx, owner, "guiyan2"); u05_recover(ctx, owner, 1); },
        // character/swd/skill.js:7500-7555
        "xianyin" =>  { u05_mark_invoked(ctx, owner, "xianyin"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:7556-7681
        "mailun" =>  { u05_mark_invoked(ctx, owner, "mailun"); u05_recover(ctx, owner, 1); u05_lose_hp(ctx, owner, 1); },
        // character/swd/skill.js:7714-7726
        "mailun5" =>  { u05_mark_invoked(ctx, owner, "mailun5"); set_flag(ctx.core, owner, "u05_mod:mailun5", 1); },
        // character/swd/skill.js:7727-7739
        "mailun6" =>  { u05_mark_invoked(ctx, owner, "mailun6"); set_flag(ctx.core, owner, "u05_mod:mailun6", 1); },
        // character/swd/skill.js:7740-7749
        "mailun7" =>  { u05_mark_invoked(ctx, owner, "mailun7"); set_flag(ctx.core, owner, "u05_mod:mailun7", 1); },
        // character/swd/skill.js:7750-7759
        "mailun8" =>  { u05_mark_invoked(ctx, owner, "mailun8"); set_flag(ctx.core, owner, "u05_mod:mailun8", 1); },
        // character/swd/skill.js:7760-7785
        "fengming" =>  { u05_mark_invoked(ctx, owner, "fengming"); if !u05_check_usable(ctx, owner, "fengming", 1) { return true; } if !u05_pay_discard(ctx, owner, "fengming", 1, true) { return true; } u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/swd/skill.js:7786-7811
        "wanjun" =>  { u05_mark_invoked(ctx, owner, "wanjun"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "nanman"), 1); },
        // character/swd/skill.js:7812-7858
        "huanling" =>  { u05_mark_invoked(ctx, owner, "huanling"); u05_turn_over(ctx, owner); },
        // character/swd/skill.js:7869-7880
        "ljifeng" =>  { u05_mark_invoked(ctx, owner, "ljifeng"); set_flag(ctx.core, owner, "u05_mod:ljifeng", 1); },
        // character/swd/skill.js:7942-7955
        "zhenjiu2" =>  { u05_mark_invoked(ctx, owner, "zhenjiu2"); u05_recover(ctx, owner, 1); },
        // character/swd/skill.js:7988-7998
        "hudun" =>  { u05_mark_invoked(ctx, owner, "hudun"); u05_change_hujia(ctx, owner, 1); },
        // character/swd/skill.js:7999-8040
        "toudan" =>  { u05_mark_invoked(ctx, owner, "toudan"); if !u05_check_usable(ctx, owner, "toudan", 1) { return true; } if !u05_pay_discard(ctx, owner, "toudan", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/swd/skill.js:8041-8106
        "shending" =>  { u05_mark_invoked(ctx, owner, "shending"); if !u05_check_usable(ctx, owner, "shending", 1) { return true; } u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:shending", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/swd/skill.js:8107-8155
        "poxiao" =>  { u05_mark_invoked(ctx, owner, "poxiao"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/swd/skill.js:1319-1335
        "lmazui3" =>  { u05_mark_invoked(ctx, owner, "lmazui3"); if !u05_pay_discard(ctx, owner, "lmazui3", 1, true) { return true; } },
        // character/swd/skill.js:2200-2215
        "huajin3" =>  { u05_mark_invoked(ctx, owner, "huajin3"); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/swd/skill.js:2583-2605
        "fuyan2" =>  { u05_mark_invoked(ctx, owner, "fuyan2"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:2874-2890
        "linyun2" =>  { u05_mark_invoked(ctx, owner, "linyun2"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "shan"), 1); },
        // character/swd/skill.js:2977-3020
        "hutian2" | "hutian3" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } set_flag(ctx.core, owner, "u05_debuff:hutian", 1); },
        // character/swd/skill.js:3289-3310
        "xuanyuan_ai" =>  { u05_mark_invoked(ctx, owner, "xuanyuan_ai"); if !u05_pay_discard(ctx, owner, "xuanyuan_ai", 1, true) { return true; } },
        // character/swd/skill.js:3364-3405
        "jilve2" | "jilve4" | "jilve5" | "jilve6" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // character/swd/skill.js:3587-3605
        "poyue2" =>  { u05_mark_invoked(ctx, owner, "poyue2"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/swd/skill.js:3963-3980
        "touxi2" =>  { u05_mark_invoked(ctx, owner, "touxi2"); let _ = u05_prey_other(ctx, owner, true); },
        // character/swd/skill.js:4023-4045
        "lianji2" | "lianji3" =>  { u05_mark_invoked(ctx, owner, skill); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/swd/skill.js:4054-4080
        "miedao1" =>  { u05_mark_invoked(ctx, owner, "miedao1"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/swd/skill.js:4228-4255
        "wangchen2" | "wangchen3" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // character/swd/skill.js:4723-4945
        "oldliaoyuan2" | "liaoyuan2" | "liaoyuan3" =>  { u05_mark_invoked(ctx, owner, skill); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/swd/skill.js:5149-5170
        "yishan2" =>  { u05_mark_invoked(ctx, owner, "yishan2"); if !u05_pay_discard(ctx, owner, "yishan2", 1, true) { return true; } },
        // character/swd/skill.js:5853-5875
        "tanlin2" =>  { u05_mark_invoked(ctx, owner, "tanlin2"); if !u05_pay_discard(ctx, owner, "tanlin2", 1, true) { return true; } },
        // character/swd/skill.js:5953-6015
        "lexue1" | "lexue2" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // character/swd/skill.js:6029-6255
        "swdtianshu2" | "swdtianshu3" | "swdtianshu_remove" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // character/swd/skill.js:6960-6980
        "guxing1" =>  { u05_mark_invoked(ctx, owner, "guxing1"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/swd/skill.js:7244-7265
        "duijue3" =>  { u05_mark_invoked(ctx, owner, "duijue3"); set_flag(ctx.core, owner, "u05_mod:duijue2", 1); },
        // character/swd/skill.js:7434-7455
        "yinguo3" =>  { u05_mark_invoked(ctx, owner, "yinguo3"); if !u05_pay_discard(ctx, owner, "yinguo3", 1, true) { return true; } },
        // character/swd/skill.js:7682-7713
        "mailun31" | "mailun32" | "mailun41" | "mailun42" =>  { u05_mark_invoked(ctx, owner, skill); set_flag(ctx.core, owner, "u05_mod:mailun5", 1); },
        // character/swd/skill.js:7859-7868
        "huanling2" =>  { u05_mark_invoked(ctx, owner, "huanling2"); set_flag(ctx.core, owner, "u05_mod:mingkong2", 1); },
        // character/yunchou/skill.js:29-107
        "diyjianxiong" =>  { u05_mark_invoked(ctx, owner, "diyjianxiong"); if !u05_pay_discard(ctx, owner, "diyjianxiong", 1, true) { return true; } },
        _ => return false,
    }
    true
}
