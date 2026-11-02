//! HP-change gates complete before the ordinary TargetDamage window.
use super::*;
pub(super) fn commit_damage_change_hp(ctx: &mut Ctx<'_>, d: &crate::event::DamageData) {
    if let Some(ydef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_YINZEI)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, ydef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                let was_enabled = ctx.core.has_skill(holder, ydef);
                if !was_enabled {
                    if let Some(inst) = ctx.core.skills.get_mut(&(holder, ydef)) {
                        inst.enabled = true;
                    }
                    ctx.log(LogEvent::SkillToggled {
                        player: holder,
                        skill: ctx.catalog.skill_name(ydef),
                        enabled: true,
                    });
                }
            }
        }
    }
    // 延妒门控（Boss 穷奇，规则 v213）：体力首次≤一半激活（隐贼同款
    // enable 直写；穷奇持延妒——与隐贼块并列， holder 独立判定）
    if let Some(dudef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_YANDU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, dudef) && !ctx.core.has_skill(holder, dudef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                if let Some(inst) = ctx.core.skills.get_mut(&(holder, dudef)) {
                    inst.enabled = true;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: holder,
                    skill: ctx.catalog.skill_name(dudef),
                    enabled: true,
                });
            }
        }
    }
    // 积财门控（Boss 饕餮，规则 v216）：体力首次≤一半激活（隐贼/延妒
    // 同款 enable 直写）
    if let Some(jdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JICAI)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, jdef) && !ctx.core.has_skill(holder, jdef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                if let Some(inst) = ctx.core.skills.get_mut(&(holder, jdef)) {
                    inst.enabled = true;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: holder,
                    skill: ctx.catalog.skill_name(jdef),
                    enabled: true,
                });
            }
        }
    }
    // 邪酬门控（Boss 相柳，规则 v223）：体力首次≤一半激活（隐贼/延妒/
    // 积财/乱常同款 enable 直写）
    if let Some(edef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ECHOU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, edef) && !ctx.core.has_skill(holder, edef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                if let Some(inst) = ctx.core.skills.get_mut(&(holder, edef)) {
                    inst.enabled = true;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: holder,
                    skill: ctx.catalog.skill_name(edef),
                    enabled: true,
                });
            }
        }
    }
    // 乱常门控（Boss 梼杌，规则 v218）：体力首次≤一半激活（隐贼/延妒/
    // 积财同款 enable 直写）
    if let Some(ldef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_LUANCHANG)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, ldef) && !ctx.core.has_skill(holder, ldef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                if let Some(inst) = ctx.core.skills.get_mut(&(holder, ldef)) {
                    inst.enabled = true;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: holder,
                    skill: ctx.catalog.skill_name(ldef),
                    enabled: true,
                });
            }
        }
    }
    // 虚失门控（Boss 朱厌，规则 v225）：体力首次≤一半激活（隐贼/延妒/
    // 积财/乱常同款 enable 直写）
    if let Some(sdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, sdef) && !ctx.core.has_skill(holder, sdef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                if let Some(inst) = ctx.core.skills.get_mut(&(holder, sdef)) {
                    inst.enabled = true;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: holder,
                    skill: ctx.catalog.skill_name(sdef),
                    enabled: true,
                });
            }
        }
    }
    if let Some(xdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XUSHI)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, xdef) && !ctx.core.has_skill(holder, xdef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                if let Some(inst) = ctx.core.skills.get_mut(&(holder, xdef)) {
                    inst.enabled = true;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: holder,
                    skill: ctx.catalog.skill_name(xdef),
                    enabled: true,
                });
            }
        }
    }
    // 炎狱门控（Boss 毕方，规则 v229）：体力首次≤一半激活（隐贼/延妒/
    // 积财/乱常/虚失同款 enable 直写）
    if let Some(ydef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_YANYU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, ydef) && !ctx.core.has_skill(holder, ydef) {
            let p = ctx.core.player(holder);
            if p.hp > 0 && p.hp <= p.max_hp as i32 / 2 {
                if let Some(inst) = ctx.core.skills.get_mut(&(holder, ydef)) {
                    inst.enabled = true;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: holder,
                    skill: ctx.catalog.skill_name(ydef),
                    enabled: true,
                });
            }
        }
    }
    shangshix_draw(ctx, d.target);
}
