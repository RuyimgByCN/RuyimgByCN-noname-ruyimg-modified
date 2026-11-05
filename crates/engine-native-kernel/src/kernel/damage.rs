use super::*;

/// Submit one canonical source amount; DamageFlow owns later callbacks and Dying.
pub(super) fn commit_damage(ctx: &mut Ctx<'_>, frame_id: EventId, mut d: crate::event::DamageData) {
    if d.prevented {
        ctx.log(LogEvent::DamagePrevented { target: d.target, presentation: None });
        finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Cancelled);
        return;
    }
    if d.amount <= 0 {
        finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Zero);
        return;
    }
    let amount = d.amount;
    let submitted = amount as u32;
    d.settlement.committed_damage = Some(submitted);
    let chain_eligible = d.nature != crate::event::DamageNature::Normal
        && !d.from_chain
        && ctx.core.player(d.target).linked;
    d.settlement.progress = crate::event::DamageProgress::HpCommitted { chain_eligible };
    // Source counters and logs consume canonical submitted event.num, never armor HP loss.
    // damageEnd num 同口径）
    if let Some(src) = d.source {
        if src != d.target {
            let p = ctx.core.player_mut(src);
            p.turn_damage = p.turn_damage.saturating_add(amount.clamp(0, 255) as u8);
        }
    }
    let mut hp_loss = submitted;
    // 护甲（G1 扩展）：1:1 抵扣伤害，先于体力扣减
    let hujia = ctx.core.player(d.target).hujia;
    if hujia > 0 {
        let absorbed = (hujia as u32).min(hp_loss);
        ctx.core.player_mut(d.target).hujia = hujia - absorbed as u8;
        hp_loss -= absorbed;
        ctx.log(LogEvent::HujiaAbsorbed {
            player: d.target,
            amount: absorbed,
        });
    }
    // 机甲血（U05 ow D.Va，规则 v141）：1:1 抵扣伤害（体力不损）——
    // 上游 changeHp 拦截（体力回补+机甲血同扣）观察等价；归零后照常
    // 扣体力（chongzhuang 重装累计登记暂缓）。
    let has_jijia = ctx
        .catalog
        .skill_by_name(crate::vocabulary::host::WUDIAN_JIJIA_KEY)
        .map(|def| ctx.core.has_skill(d.target, def))
        .unwrap_or(false);
    if has_jijia {
        let j = ctx.core.player(d.target).jijia_hp;
        if j > 0 {
            let absorbed = (j as u32).min(hp_loss);
            ctx.core.player_mut(d.target).jijia_hp = j - absorbed as u8;
            hp_loss -= absorbed;
            ctx.log(LogEvent::HujiaAbsorbed {
                player: d.target,
                amount: absorbed,
            });
        }
    }
    let receipt = format!("vm_damage_receipt:{frame_id}");
    for ((_, key), value) in &mut ctx.core.vm_aux.flags {
        if key == &receipt {
            *value = d.target as u32 + 1;
        }
    }
    let p = ctx.core.player_mut(d.target);
    // 无限体力（上游 Infinity）：伤害照常结算与记录，但不改变体力
    //（hp 保持 == max_hp，故「已受伤」类判据恒假——与上游一致）。
    if !p.infinite_hp() {
        let Some(hp) = p.hp.checked_sub(hp_loss as i32) else {
            ctx.res.fault = Some(crate::machine::VmFault {
                skill: "intrinsic_damage".into(),
                owner: d.target,
                event: "Damage".into(),
                kind: "DamageHpOverflow".into(),
                detail: "HP subtraction exceeds i32".into(),
            });
            return;
        };
        p.hp = hp;
    }
    d.settlement.hp_loss = if p.infinite_hp() { 0 } else { hp_loss };
    ctx.res.frame_mut(frame_id).expect("damage root").data = EventData::Damage(d.clone());
    ctx.log(LogEvent::DamageDealt {
        source: d.source,
        target: d.target,
        amount: d.amount.max(0) as u32,
        redirected_from: d.redirect_chain.clone(),
        card: d.card, presentation: None });
    if d.nature != crate::event::DamageNature::Normal && ctx.core.player(d.target).linked {
        ctx.core.player_mut(d.target).linked = false;
        ctx.log(LogEvent::Linked {
            player: d.target,
            linked: false,
        });
    }

    ctx.res.frame_mut(frame_id).expect("damage root").data = EventData::Damage(d.clone());
    hlss_light_refill(ctx, d.target);
    damage_change_hp::commit_damage_change_hp(ctx, &d);
}

pub(super) fn post_damage(
    ctx: &mut Ctx<'_>,
    d: crate::event::DamageData,
    effect: crate::event::NativeDamageEffect,
    owner: PlayerId,
) {
    if effect == crate::event::NativeDamageEffect::Shuchui {
        ctx.core.turn.shuchui_hit = true;
    }
    if effect == crate::event::NativeDamageEffect::Paotai {
        let level = u32::from(ctx.core.player(d.target).paotai_lv);
        let amount = u32::try_from(d.amount).expect("positive admitted quantity");
        ctx.core.player_mut(d.target).paotai_lv =
            u8::try_from(level.saturating_sub(amount)).expect("bounded level");
    }
    if effect == crate::event::NativeDamageEffect::Xixue {
        let source = d.source.expect("admitted source role");
        ctx.recover(source, d.amount as u32, None);
    }

    if effect == crate::event::NativeDamageEffect::Yinzei {
        let source = d.source.expect("admitted source");
        let pool: Vec<_> = ctx
            .core
            .player(source)
            .hand
            .iter()
            .chain(ctx.core.player(source).equips.iter())
            .copied()
            .collect();
        let selected = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
        ctx.take_owned_card(source, selected);
        ctx.to_discard(selected);
        ctx.log(LogEvent::CardDiscarded {
            player: source,
            cards: vec![selected],
        });
        ctx.emit(EventData::CardsDiscarded {
            player: source,
            cards: vec![selected],
        });
    }

    if effect == crate::event::NativeDamageEffect::Zlfanshi {
        if ctx.core.player(owner).zlfanshi_terra {
            let targets: Vec<_> = ctx
                .core
                .seat_ring
                .iter()
                .copied()
                .filter(|p| *p != owner && ctx.core.player(*p).alive)
                .collect();
            if !targets.is_empty() {
                let target = targets[ctx.core.rng.gen_range(targets.len() as u32) as usize];
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    target,
                    1,
                )));
            }
        } else {
            ctx.core.player_mut(owner).zlfanshi_terra = true;
        }
    }
    if effect == crate::event::NativeDamageEffect::Zhue {
        // 逐厄（Boss 穷奇，规则 v211）：global damageEnd——任何人受到伤害
        // 后（来源存活且非穷奇），穷奇与来源各摸一张（上游 boss_zhue
        // asyncDraw([player, source]) 同口径）
        if let Some(_zdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_ZHUE)
        {
            // 任一存活穷奇持有者（单 boss 场景恰一）
            let holders: Vec<PlayerId> = vec![owner];
            for holder in holders {
                if !ctx.core.player(holder).alive {
                    continue;
                }
                if let Some(src) = d.source {
                    if src != holder && ctx.core.player(src).alive {
                        ctx.start_draw(holder, 1);
                        ctx.start_draw(src, 1);
                    }
                }
            }
        }
    }
    if effect == crate::event::NativeDamageEffect::Abi {
        let source = d.source.expect("admitted source");
        let nature = [
            crate::event::DamageNature::Fire,
            crate::event::DamageNature::Thunder,
        ][ctx.core.rng.gen_range(2) as usize];
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            source,
            1,
            nature,
            None,
        )));
    }

    if effect == crate::event::NativeDamageEffect::Lingqu {
        // 灵渠（Boss 玄武，规则 v237）：每次受到伤害后摸 1 张牌且受击计数
        // +1（上游 boss_lingqu damageEnd draw+storage++ 同口径；零量由 Zero/Stopped 截断；
        // 计数=PlayerState.lingqu_count 跨回合持久，手牌上限 +X）
        if let Some(ldef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_LINGQU)
        {
            if ctx.core.has_skill(d.target, ldef) && ctx.core.player(d.target).alive {
                ctx.start_draw(d.target, 1);
                let p = ctx.core.player_mut(d.target);
                p.lingqu_count = p.lingqu_count.saturating_add(1);
            }
        }
    }

    if effect == crate::event::NativeDamageEffect::Renxing {
        // 忍性（年兽形态，规则 v249）：global damageEnd——持有者非当前回
        // 合角色时，任意角色受到伤害后其摸 1（上游 boss_renxing damageEnd
        // global `_status.currentPhase != player` 同口径）
        if let Some(_rxdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_RENXING)
        {
            let holders: Vec<PlayerId> = vec![owner];
            for holder in holders {
                ctx.start_draw(holder, 1);
            }
        }
    }
    if effect == crate::event::NativeDamageEffect::Cuidui {
        // 催毒（Boss 书精，规则 v220）：来源持催毒且目标非己、无中毒 →
        // 目标授中毒实例（运行时授技=skills.insert；木神勾芒不在目录跳过
        // 联动摸牌；上游 boss_cuidui damageEnd addSkill 同口径）
        if let Some(cdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_CUIDUI)
        {
            let zdef = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_ZHONGDU);
            if let (Some(src), Some(zd)) = (d.source, zdef) {
                if src != d.target
                    && ctx.core.has_skill(src, cdef)
                    && ctx.core.player(src).alive
                    && ctx.core.player(d.target).alive
                    && !ctx.core.has_skill_any_state(d.target, zd)
                {
                    for skdef in [zd] {
                        ctx.core.skills.insert(
                            (d.target, skdef),
                            crate::state::SkillInstance {
                                owner: d.target,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                        ctx.log(LogEvent::SkillToggled {
                            player: d.target,
                            skill: ctx.catalog.skill_name(skdef),
                            enabled: true,
                        });
                    }
                }
            }
        }
        let named = ctx.catalog.mushengoumang_general_id();
        if let Some(holder) = ctx.core.seat_ring.iter().copied().find(|p| {
            ctx.core.player(*p).alive && ctx.core.player(*p).general_slots[0] == Some(named)
        }) {
            ctx.start_draw(holder, 1);
        }
    }
    if effect == crate::event::NativeDamageEffect::Duqu {
        let source = d.source.expect("admitted source");
        let player = ctx.core.player_mut(source);
        player.shedu = player.shedu.saturating_add(1);
    }
}

/// 禳星·光（T11 Boss 诸葛亮族批 115，规则 v303）：持有「禳星」且星未
/// 归零（技能实例启用）时，任意体力变化后回满体力（上游 `hlss_xiangxing_hp`
/// 子技 changeHp/loseMaxHp/gainMaxHp → `hp = maxHp` 同口径；在伤害/失去
/// 体力提交点、濒死判定前调用，故星未耗尽者不会进入濒死）。
pub(crate) fn hlss_light_refill(ctx: &mut Ctx<'_>, player: PlayerId) {
    let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::host::HLSS_XIANGXING_KEY)
    else {
        return;
    };
    let p = ctx.core.player_mut(player);
    if !p.alive || p.hp >= p.max_hp as i32 {
        return;
    }
    let enabled = ctx
        .core
        .skills
        .get(&(player, def))
        .map(|i| i.enabled)
        .unwrap_or(false);
    if !enabled {
        return;
    }
    let max = ctx.core.player(player).max_hp as i32;
    ctx.core.player_mut(player).hp = max;
}

/// 护天归还（swd 批 131，规则 v314）：把持有者寄存于目标处的牌归还目标，
/// 清理双方寄存状态（上游 `hutian3` 收束同口径）。
pub(crate) fn hutian_return_deposit(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let Some(target) = ctx.core.player_mut(owner).hutian_deposit.take() else {
        return;
    };
    let cards = std::mem::take(&mut ctx.core.player_mut(target).hutian_hold);
    ctx.core.player_mut(target).hutian_owner = None;
    if !cards.is_empty() {
        for c in cards.iter().copied() {
            ctx.core.player_mut(target).hand.push(c);
        }
        ctx.log(LogEvent::CardsGiven {
            from: owner,
            to: target,
            cards,
        });
    }
    ctx.core.player_mut(owner).hutian_just_given = false;
}

/// 失去体力提交（逐技对照批三 v77）：直接扣减体力——不经护甲抵扣、
/// 无伤害前置窗口/铁索传导/伤害来源触发技（LoseHp 是独立事件种）；
/// 体力 ≤0 与伤害同口径进入濒死（上游 loseHp 语义）。
pub(crate) fn commit_lose_hp(ctx: &mut Ctx<'_>, player: PlayerId, mut amount: u32) {
    if amount == 0 {
        return;
    }
    // 护天护体（swd 批 131，规则 v314）：失去体力同样以「体力降至寄存数」
    // 为上限（上游 hutian2 loseHpBegin 同口径）
    {
        let hold = ctx.core.player(player).hutian_hold.len() as i32;
        if hold > 0 {
            let hp = ctx.core.player(player).hp;
            let cap = hp - hold;
            if cap < amount as i32 {
                amount = cap.max(0) as u32;
            }
        }
    }
    if amount == 0 {
        ctx.log(LogEvent::HpLost { player, amount: 0 });
        return;
    }
    let p = ctx.core.player_mut(player);
    if !p.infinite_hp() {
        p.hp -= amount as i32;
    }
    hlss_light_refill(ctx, player);
    ctx.log(LogEvent::HpLost { player, amount });
    let entering = ctx.core.player(player).hp <= 0 && !ctx.core.player(player).dying;
    if entering {
        ctx.core.player_mut(player).dying = true;
        ctx.log(LogEvent::DyingEntered { player });
        ctx.emit(EventData::Dying {
            player,
            source: None,
        });
    }
    // 伤逝（规则 v260）：体力变化（失去）后补牌
    shangshix_draw(ctx, player);
}

/// 伤逝（Boss 张春华，规则 v260）补牌半：
pub(super) fn shangshix_draw(ctx: &mut Ctx<'_>, player: PlayerId) {
    if let Some(ssdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_SHANGSHIX)
    {
        if ctx.core.player(player).alive && ctx.core.has_skill(player, ssdef) {
            let n = ctx.core.player(player).hand.len();
            if n < 4 {
                ctx.start_draw(player, (4 - n) as u32);
            }
        }
    }
}

pub(super) fn finish_damage(
    ctx: &mut Ctx<'_>,
    root: EventId,
    mut d: crate::event::DamageData,
    outcome: crate::event::DamageOutcome,
) {
    d.settlement.outcome = outcome;
    // The active window/child keeps its progress until the root resumes.

    ctx.res.frame_mut(root).expect("damage root").data = EventData::Damage(d);
}
