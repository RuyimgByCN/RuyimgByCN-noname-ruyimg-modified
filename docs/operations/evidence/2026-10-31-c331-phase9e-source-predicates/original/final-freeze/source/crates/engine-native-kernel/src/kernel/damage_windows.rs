//! Bodies of admitted native source units; the window commit performs no skill effects.
use super::*;
use crate::event::DamageWindowKind as Window;
pub(super) fn apply_native_damage(
    ctx: &mut Ctx<'_>,
    root: EventId,
    window: Window,
    effect: crate::event::NativeDamageEffect,
) {
    let mut d = ctx.res.frame(root).expect("damage root").damage().clone();
    if d.prevented
        || d.amount <= 0
        || d.settlement.outcome != crate::event::DamageOutcome::Running
        || d.settlement.notrigger
    {
        return;
    }
    let mut amount = i64::from(d.amount);
    match window {
        Window::Before => {
            if effect == crate::event::NativeDamageEffect::Jidong {
                // 急冻冰封（U05 ow 美，规则 v127）：锁定——冰封目标受所有伤害无
                // 效（上游 jidong2 damageBefore trigger.cancel() 同口径；铁索传导
                // 的独立事件同门控）。置于各伤害增幅钩前：伤害已整体取消，
                // damageBegin 类增幅同上游一样不再结算。
                if ctx.core.player(d.target).jidong_active {
                    ctx.log(LogEvent::DamagePrevented { target: d.target });
                    finish_damage(ctx, root, d, crate::event::DamageOutcome::Cancelled);
                    return;
                }
            }
            if effect == crate::event::NativeDamageEffect::Chengxin {
                // 诚信免伤（U05 xianjian 明绣，规则 v164）：锁定——免伤状态目标受
                // 所有伤害无效（上游 chengxin2 damageBefore trigger.cancel 同口径）
                if ctx.core.player(d.target).chengxin_active {
                    ctx.log(LogEvent::DamagePrevented { target: d.target });
                    finish_damage(ctx, root, d, crate::event::DamageOutcome::Cancelled);
                    return;
                }
            }
        }
        Window::Begin => {
            if effect == crate::event::NativeDamageEffect::Chilongya {
                // 赤龙牙（U05 swd 第四批，规则 v88）：锁定——来源装备赤龙牙时
                // 火属性伤害 +1（上游 damageBegin num++；非铁索传导照常）
                if d.nature == crate::event::DamageNature::Fire && !d.from_chain {
                    if let Some(src) = d.source {
                        if ctx.core.player(src).alive
                            && ctx
                                .core
                                .player(src)
                                .equips
                                .iter()
                                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Chilongya)
                        {
                            amount += 1;
                        }
                    }
                }
            }
            if effect == crate::event::NativeDamageEffect::Xingjunyan {
                amount += 1;
            }
            if effect == crate::event::NativeDamageEffect::Liangou {
                // 链钩（U05 ow 路霸，规则 v128）：本回合被钩者首次受伤害 +1
                //（上游 liangou2 damageBegin trigger.num++ usable:1 同口径；来源
                // 不限=上游无 source 条件；仅路霸出牌阶段可钩住，换回合解除）
                if let Some(t) = ctx.core.turn.liangou_hook {
                    if d.target == t && !ctx.core.turn.liangou_bonus_used {
                        amount += 1;
                        ctx.core.turn.liangou_bonus_used = true;
                    }
                }
            }
            if effect == crate::event::NativeDamageEffect::Feiren {
                // 飞刃（U05 ow 源氏，规则 v129）：锁定——你的黑桃杀伤害 +1（非
                // 铁索传导；上游 damageBegin trigger.num++ notLink 同口径；花色按
                // 实体牌——火/雷杀为黑桃时同样命中）
                if d.card
                    .map(|c| {
                        ctx.catalog.card_kind(c).is_sha_like()
                            && ctx.catalog.cards[c as usize].suit == crate::catalog::Suit::Spade
                    })
                    .unwrap_or(false)
                    && !d.from_chain
                {
                    if let Some(src) = d.source {
                        let has_feiren = ctx
                            .catalog
                            .skill_by_name(crate::vocabulary::host::WUDIAN_FEIREN_KEY)
                            .map(|def| ctx.core.has_skill(src, def))
                            .unwrap_or(false);
                        if has_feiren && ctx.core.player(src).alive {
                            amount += 1;
                        }
                    }
                }
            }
            if effect == crate::event::NativeDamageEffect::Duwen {
                // 毒瘟（U07 ow，规则 v122）：锁定——造成伤害时若你手牌数与受伤害
                // 角色相等，伤害 +1（上游 duwen damageBegin num++ 同口径；铁索传
                // 导为独立事件照常判定）
                if let Some(src) = d.source {
                    if ctx.core.player(src).alive
                        && ctx.core.player(src).hand.len() == ctx.core.player(d.target).hand.len()
                    {
                        if let Some(def) = ctx
                            .catalog
                            .skill_by_name(crate::vocabulary::host::WUDIAN_DUWEN_KEY)
                        {
                            if ctx.core.has_skill(src, def) {
                                amount += 1;
                            }
                        }
                    }
                }
            }
            if matches!(
                effect,
                crate::event::NativeDamageEffect::MailunOut3
                    | crate::event::NativeDamageEffect::MailunOut4
            ) {
                let src = d.source.expect("source role");
                amount += if effect == crate::event::NativeDamageEffect::MailunOut3 {
                    -1
                } else {
                    1
                };
                ctx.core.player_mut(src).mailun_out_used = true;
            }
            if matches!(
                effect,
                crate::event::NativeDamageEffect::MailunIn3
                    | crate::event::NativeDamageEffect::MailunIn4
            ) {
                amount += if effect == crate::event::NativeDamageEffect::MailunIn3 {
                    -1
                } else {
                    1
                };
                ctx.core.player_mut(d.target).mailun_in_used = true;
            }
            if effect == crate::event::NativeDamageEffect::Hutian {
                // 护天护体（swd 批 131，规则 v314）：目标寄存护天牌时伤害以「体力降至
                // 寄存数」为上限（上游 hutian2 damageBegin priority -55 `trigger.num =
                // hp - storage.hutian2.length` 同口径）
                {
                    let hold = ctx.core.player(d.target).hutian_hold.len();
                    if hold > 0 {
                        let Ok(hold) = i64::try_from(hold) else {
                            ctx.fail_damage_amount();
                            return;
                        };
                        let Some(cap) = i64::from(ctx.core.player(d.target).hp).checked_sub(hold)
                        else {
                            ctx.fail_damage_amount();
                            return;
                        };
                        amount = amount.min(cap);
                    }
                }
            }
        }
        Window::Begin1 => {
            if effect == crate::event::NativeDamageEffect::Ciqiu {
                let source = d.source.expect("admitted source role");
                amount += 1;
                if amount >= i64::from(ctx.core.player(d.target).hp) {
                    ctx.core.turn.ciqiu_dying.push(d.target);
                    let def = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::host::WUDIAN_CIQIU_KEY)
                        .expect("immutable Ciqiu definition");
                    ctx.core.skills.remove(&(source, def));
                }
            }
            if effect == crate::event::NativeDamageEffect::Huoshen {
                // 火神（Boss 周瑜，规则 v263）：锁定——火属性伤害对你无效，你回复
                // 1 点体力（上游 damageBegin1 cancel+recover() 无参=1）。
                if d.nature == crate::event::DamageNature::Fire {
                    if let Some(hsdef) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_HUOSHEN)
                    {
                        if ctx.core.player(d.target).alive && ctx.core.has_skill(d.target, hsdef) {
                            ctx.log(LogEvent::DamagePrevented { target: d.target });
                            finish_damage(
                                ctx,
                                root,
                                d.clone(),
                                crate::event::DamageOutcome::Cancelled,
                            );
                            ctx.recover(d.target, 1, None);
                            return;
                        }
                    }
                }
            }
            if effect == crate::event::NativeDamageEffect::NitaiOut {
                // 拟态（Boss 梼杌，规则 v217）：锁定——你于回合外受到的火属性伤害
                // +1（上游 boss_nitai_out damageBegin1 num++ 同口径）
                if d.nature == crate::event::DamageNature::Fire {
                    if let Some(ndef) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_NITAI)
                    {
                        if ctx.core.has_skill(d.target, ndef) && ctx.core.turn.current != d.target {
                            amount += 1;
                        }
                    }
                }
            }
        }
        Window::Begin4 => {
            if effect == crate::event::NativeDamageEffect::Zhaohuo {
                // 招祸（Boss 毕方，规则 v227）：锁定——你免疫所有火属性伤害（上游
                // boss_zhaohuo damageBegin4 trigger.cancel 的火伤分支同口径；诚信免
                // 伤同位）
                if d.nature == crate::event::DamageNature::Fire {
                    if let Some(zdef) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_ZHAOHUO)
                    {
                        if ctx.core.has_skill(d.target, zdef) {
                            ctx.log(LogEvent::DamagePrevented { target: d.target });
                            finish_damage(ctx, root, d, crate::event::DamageOutcome::Cancelled);
                            return;
                        }
                    }
                }
            }
            if effect == crate::event::NativeDamageEffect::NitaiIn {
                // 拟态（Boss 梼杌，规则 v217）：锁定——防止你于回合内受到的伤害
                //（上游 boss_nitai_in damageBegin4 trigger.cancel 同口径：当前回合
                // =持有者）
                if let Some(ndef) = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::BOSS_NITAI)
                {
                    if ctx.core.has_skill(d.target, ndef) && ctx.core.turn.current == d.target {
                        ctx.log(LogEvent::DamagePrevented { target: d.target });
                        finish_damage(ctx, root, d, crate::event::DamageOutcome::Cancelled);
                        return;
                    }
                }
            }
            if effect == crate::event::NativeDamageEffect::Bufo {
                // 灵渠（Boss 玄武，规则 v237）：受到的大于 1 点伤害视为 0 点（上游
                // 不佛（Boss 地藏王，规则 v274）：受到大于 1 点的伤害时改为其中 1
                // 点（上游 boss_bufo damageBegin4 num>1→num-1 同口径；置于灵渠归零
                // 检查前——灵渠读调整后终值；两技同持为不同武将不共存，序未登记）
                if amount > 1 {
                    if let Some(bfdef) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_BUFO)
                    {
                        if ctx.core.has_skill(d.target, bfdef) {
                            amount -= 1;
                        }
                    }
                }
            }
            if effect == crate::event::NativeDamageEffect::LingquCancel {
                // boss_lingqu cancel damageBegin4 num>1 → num=0 同口径；置于增幅后
                // 统计前——终值>1 才归零；随后 Zero/Stopped 不派普通 damageEnd。）
                if amount > 1 {
                    if let Some(ldef) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_LINGQU)
                    {
                        if ctx.core.has_skill(d.target, ldef) {
                            amount = 0;
                        }
                    }
                }
            }
        }
        _ => return,
    }
    let Ok(amount) = i32::try_from(amount) else {
        ctx.res.fault = Some(crate::machine::VmFault {
            skill: "intrinsic_damage".into(),
            owner: d.target,
            event: "Damage".into(),
            kind: "DamageAmountOverflow".into(),
            detail: "canonical source amount exceeds i32".into(),
        });
        return;
    };
    d.amount = amount;
    ctx.res.frame_mut(root).unwrap().data = EventData::Damage(d);
}
