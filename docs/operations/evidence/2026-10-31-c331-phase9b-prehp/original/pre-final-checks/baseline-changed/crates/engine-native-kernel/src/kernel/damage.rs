use super::*;

/// Submit one canonical source amount; DamageFlow owns later callbacks and Dying.
pub(super) fn commit_damage(ctx: &mut Ctx<'_>, frame_id: EventId, mut d: crate::event::DamageData) {
    if d.prevented {
        ctx.log(LogEvent::DamagePrevented { target: d.target });
        finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Cancelled);
        return;
    }
    if d.amount <= 0 {
        finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Zero);
        return;
    }
    // 绝情（Boss 张春华，规则 v260；一将包旧绝情）：锁定——你造成的伤害
    // 改为目标失去等量体力（上游 source damageBefore cancel+loseHp(num)
    // 同口径；num=增幅前原始值=d.amount；取消后伤害事件不再结算=受击/
    // 加成类不触发；置于急冻前——上游 damageBefore 同层内容序）
    if let Some(jqdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::JUE_QING)
    {
        if let Some(src) = d.source {
            if src != d.target && ctx.core.player(src).alive && ctx.core.has_skill(src, jqdef) {
                ctx.emit(EventData::LoseHp {
                    player: d.target,
                    amount: d.amount as u32,
                });
                finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::ReplacedByLoseHp);
        return;
            }
        }
    }
    // 无心（Boss 张春华，规则 v260）伤害半：锁定——你受到的伤害改为失去
    // 1 点体力（上游 damageBefore priority 10 cancel+loseHp() 无参=1 同
    // 口径；置于绝情后=同层 priority 序）
    if let Some(wxdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_WUXIN)
    {
        if ctx.core.has_skill(d.target, wxdef) && ctx.core.player(d.target).alive {
            ctx.emit(EventData::LoseHp {
                player: d.target,
                amount: 1,
            });
            finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::ReplacedByLoseHp);
        return;
        }
    }
    // 急冻冰封（U05 ow 美，规则 v127）：锁定——冰封目标受所有伤害无
    // 效（上游 jidong2 damageBefore trigger.cancel() 同口径；铁索传导
    // 的独立事件同门控）。置于各伤害增幅钩前：伤害已整体取消，
    // damageBegin 类增幅同上游一样不再结算。
    if ctx.core.player(d.target).jidong_active {
        ctx.log(LogEvent::DamagePrevented { target: d.target });
        finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Cancelled);
        return;
    }
    // 诚信免伤（U05 xianjian 明绣，规则 v164）：锁定——免伤状态目标受
    // 所有伤害无效（上游 chengxin2 damageBefore trigger.cancel 同口径）
    if ctx.core.player(d.target).chengxin_active {
        ctx.log(LogEvent::DamagePrevented { target: d.target });
        finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Cancelled);
        return;
    }
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
                finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Cancelled);
        return;
            }
        }
    }
    // 火神（Boss 周瑜，规则 v263）：锁定——火属性伤害对你无效，你回复
    // 1 点体力（上游 damageBegin1 cancel+recover() 无参=1 同口径；置于
    // 招祸同位=damageBegin 层，取消后增幅不结算）
    if d.nature == crate::event::DamageNature::Fire {
        if let Some(hsdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_HUOSHEN)
        {
            if ctx.core.player(d.target).alive && ctx.core.has_skill(d.target, hsdef) {
                ctx.log(LogEvent::DamagePrevented { target: d.target });
                ctx.recover(d.target, 1, None);
                finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Cancelled);
        return;
            }
        }
    }
    // 拟态（Boss 梼杌，规则 v217）：锁定——防止你于回合内受到的伤害
    //（上游 boss_nitai_in damageBegin4 trigger.cancel 同口径：当前回合
    // =持有者）
    if let Some(ndef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_NITAI)
    {
        if ctx.core.has_skill(d.target, ndef) && ctx.core.turn.current == d.target {
            ctx.log(LogEvent::DamagePrevented { target: d.target });
            finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Cancelled);
        return;
        }
    }
    let mut amount = i64::from(d.amount);
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
    // 赤龙牙（U05 swd 第四批，规则 v88）：锁定——来源装备赤龙牙时
    // 火属性伤害 +1（上游 damageBegin num++；非铁索传导照常）
    if d.nature == crate::event::DamageNature::Fire {
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
    // 星君眼（U05 swd 第九批，规则 v93）：锁定——你的【杀】伤害 +1
    //（上游 damageBegin num++；card==sha 同口径；铁索传导为独立事件
    // 照常 +1）
    if d.card
        .map(|c| ctx.catalog.card_kind(c) == CardKind::Sha)
        .unwrap_or(false)
    {
        if let Some(src) = d.source {
            if ctx.core.player(src).alive
                && ctx
                    .core
                    .player(src)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::Xingjunyan)
            {
                amount += 1;
            }
        }
    }
    // 链钩（U05 ow 路霸，规则 v128）：本回合被钩者首次受伤害 +1
    //（上游 liangou2 damageBegin trigger.num++ usable:1 同口径；来源
    // 不限=上游无 source 条件；仅路霸出牌阶段可钩住，换回合解除）
    if let Some(t) = ctx.core.turn.liangou_hook {
        if d.target == t && !ctx.core.turn.liangou_bonus_used {
            amount += 1;
            ctx.core.turn.liangou_bonus_used = true;
        }
    }
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
    // 刺猬（U07 玩点论杀韩龙，规则 v111）：锁定——你的杀对满血目标伤
    // 害 +1；加后 ≥ 目标体力则目标获本回合濒死即死标记（上游
    // addTempSkill("ciqiu_dying")）且本技能移除（一次性）
    if d.card
        .map(|c| ctx.catalog.card_kind(c) == CardKind::Sha)
        .unwrap_or(false)
    {
        if let Some(src) = d.source {
            if ctx.core.player(d.target).alive
                && ctx.core.player(d.target).hp == ctx.core.player(d.target).max_hp as i32
            {
                if let Some(def) = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::host::WUDIAN_CIQIU_KEY)
                {
                    if ctx.core.has_skill(src, def) {
                        amount += 1;
                        if amount >= i64::from(ctx.core.player(d.target).hp) {
                            ctx.core.turn.ciqiu_dying.push(d.target);
                            ctx.core.skills.remove(&(src, def));
                        }
                    }
                }
            }
        }
    }
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
    // 回合伤害统计（U05 xianjian 明绣，规则 v160 灵火门控）：来源本回
    // 合已造成伤害 +实际量（护甲/机甲吸收后；上游 getStat("damage") 按
    // 醉酒（Boss 白无常，规则 v255）：你使用的【杀】造成的伤害 +1（上
    // 游 damageBegin1 sha+shaParent 同口径；火杀/雷杀同属 sha 族）
    if d.card
        .map(|c| {
            matches!(
                ctx.catalog.card_kind(c),
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
            )
        })
        .unwrap_or(false)
    {
        if let Some(zjdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_ZUIJIU)
        {
            if let Some(src) = d.source {
                if ctx.core.has_skill(src, zjdef) {
                    amount += 1;
                }
            }
        }
    }
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
    // boss_lingqu cancel damageBegin4 num>1 → num=0 同口径；置于增幅后
    // 统计前——终值>1 才归零；0 点伤害仍完成事件→受击摸 1/计数照常）
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
    // 脉轮（swd 批 137，规则 v316）：3=首次造成/受到伤害 -1、4=+1（各自
    // 一次性；上游 mailun31/32/41/42 damageBegin 用后移除同口径）
    {
        if let Some(src) = d.source {
            let mode = ctx.core.player(src).mailun_mode;
            if (mode == 3 || mode == 4) && !ctx.core.player(src).mailun_out_used {
                amount = if mode == 3 {
                    amount.saturating_sub(1)
                } else {
                    amount.saturating_add(1)
                };
                ctx.core.player_mut(src).mailun_out_used = true;
            }
        }
        let mode = ctx.core.player(d.target).mailun_mode;
        if (mode == 3 || mode == 4) && !ctx.core.player(d.target).mailun_in_used {
            amount = if mode == 3 {
                amount.saturating_sub(1)
            } else {
                amount.saturating_add(1)
            };
            ctx.core.player_mut(d.target).mailun_in_used = true;
        }
    }
    // 护天护体（swd 批 131，规则 v314）：目标寄存护天牌时伤害以「体力降至
    // 寄存数」为上限（上游 hutian2 damageBegin priority -55 `trigger.num =
    // hp - storage.hutian2.length` 同口径）
    {
        let hold = ctx.core.player(d.target).hutian_hold.len() as i32;
        if hold > 0 {
            let hp = ctx.core.player(d.target).hp;
            let cap = hp - hold;
            if i64::from(cap) < amount {
                amount = i64::from(cap.max(0));
            }
        }
    }
    let Ok(amount)=i32::try_from(amount) else {
        ctx.res.fault=Some(crate::machine::VmFault{skill:"intrinsic_damage".into(),owner:d.target,event:"Damage".into(),kind:"DamageAmountOverflow".into(),detail:"canonical source amount exceeds i32".into()});return;
    };
    d.amount = amount;
    if amount <= 0 { finish_damage(ctx, frame_id, d, crate::event::DamageOutcome::Zero); return; }
    let submitted = amount as u32;
    d.settlement.committed_damage = Some(submitted);
    d.settlement.progress = crate::event::DamageProgress::HpCommitted;
    // Source counters and logs consume canonical submitted event.num, never armor HP loss.
    // damageEnd num 同口径）
    if let Some(src) = d.source {
        if src != d.target {
            let p = ctx.core.player_mut(src);
            p.turn_damage = p.turn_damage.saturating_add(amount.clamp(0,255) as u8);
        }
    }
    // 还侠跟踪（swd 屈娴，批 27）：还侠转化杀造成真实伤害 → 置位（上
    // 游 expire 子技 source damageAfter 同口径；牌 id 匹配最后提交者）
    if let Some(dcard) = d.card {
        if let Some((card, caused)) = ctx.core.turn.huanxia_track.last_mut() {
            if *card == dcard {
                *caused = true;
            }
        }
    }
    // 鼠槌（swd 卡牌面窗口机批）：多杀循环链武装中，任一杀造成真实伤害
    // → 置位（上游 shuchui2 charlotte source damageEnd filter card=sha
    // 同口径；武装=当前回合玩家=链主）
    if ctx.core.turn.shuchui_armed
        && d.source == Some(ctx.core.turn.current)
        && d.card
            .map(|c| ctx.catalog.card_kind(c).is_sha_like())
            .unwrap_or(false)
    {
        ctx.core.turn.shuchui_hit = true;
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
        let Some(hp)=p.hp.checked_sub(hp_loss as i32) else {
            ctx.res.fault=Some(crate::machine::VmFault{skill:"intrinsic_damage".into(),owner:d.target,event:"Damage".into(),kind:"DamageHpOverflow".into(),detail:"HP subtraction exceeds i32".into()});return;
        };
        p.hp = hp;
    }
    d.settlement.hp_loss = if p.infinite_hp() { 0 } else { hp_loss };
    ctx.res.frame_mut(frame_id).expect("damage root").data = EventData::Damage(d.clone());
    hlss_light_refill(ctx, d.target);
    ctx.log(LogEvent::DamageDealt {
        source: d.source,
        target: d.target,
        amount: d.amount.max(0) as u32,
        redirected_from: d.redirect_chain.clone(),
        card: d.card,
    });
}

pub(super) fn post_damage(ctx: &mut Ctx<'_>, d: crate::event::DamageData) {
    // 炮台降级（U05 ow，规则 v123）：炮台持有者受伤 1 点降 1 级（上游
    // paotai3 damageEnd forced：storage -= trigger.num；下限 0）
    if ctx.core.player(d.target).paotai_lv > 0 && d.amount > 0 && !d.prevented {
        let lv = ctx.core.player(d.target).paotai_lv;
        ctx.core.player_mut(d.target).paotai_lv = lv.saturating_sub(d.amount as u8);
    }
    // 吸血鬼指环（U05 swd 第六批，规则 v90）：锁定——你的【杀】造成
    // 伤害后，若你已受伤，回复等量体力（上游 xixue：damageEnd sha &&
    // hp<maxHp → recover(num)）
    if let Some(src) = d.source {
        if src != d.target
            && ctx.core.player(src).alive
            && ctx.core.player(src).hp < ctx.core.player(src).max_hp as i32
            && d.card
                .map(|c| ctx.catalog.card_kind(c) == CardKind::Sha)
                .unwrap_or(false)
            && ctx
                .core
                .player(src)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Xixueguizhihuan)
        {
            ctx.recover(src, d.amount.max(0) as u32, None);
        }
    }
    // 青龙令珠（U05 swd 第九批，规则 v93）：你对其他角色造成属性伤害
    // 后（目标存活），获得其一张牌（上游 damageEnd hasNature + alive →
    // gainPlayerCard he；RNG 口径同批四代抽）
    if d.nature != crate::event::DamageNature::Normal {
        if let Some(src) = d.source {
            let t = d.target;
            if src != t
                && ctx.core.player(src).alive
                && ctx.core.player(t).alive
                && ctx
                    .core
                    .player(src)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qinglonglingzhu)
            {
                let he_len = ctx.core.player(t).hand.len() + ctx.core.player(t).equips.len();
                if he_len > 0 {
                    let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                    let hand_n = ctx.core.player(t).hand.len();
                    let c = if pick < hand_n {
                        ctx.core.player(t).hand[pick]
                    } else {
                        ctx.core.player(t).equips[pick - hand_n]
                    };
                    if ctx.core.player(t).hand.contains(&c) {
                        ctx.core.player_mut(t).hand.retain(|x| *x != c);
                    } else {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                    }
                    ctx.core.player_mut(src).hand.push(c);
                    ctx.log(LogEvent::CardsGiven {
                        from: t,
                        to: src,
                        cards: vec![c],
                    });
                }
            }
        }
    }
    // 隐贼（Boss 混沌，规则 v210）双臂：①激活门控——体力首次减至一半
    // 或更少时启用（SkillInstance.enabled 直写+SkillToggled；上游
    // boss_yinzei_switch changeHp enableSkill 同口径）；②伤害后弃牌——
    // 已激活且持有者无手牌 → 来源弃置一张手牌（确定性取首张；上游
    // damageEnd randomDiscard 随机改首张登记差异）
    if let Some(ydef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_YINZEI)
    {
        let holder = d.target;
        let has_yinzei = ctx.core.has_skill_any_state(holder, ydef);
        if has_yinzei {
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
            // 伤害后弃牌（激活态）
            if ctx.core.has_skill(holder, ydef)
                && ctx.core.player(holder).hand.is_empty()
                && ctx.core.player(holder).hp > 0
            {
                if let Some(src) = d.source {
                    if src != holder
                        && ctx.core.player(src).alive
                        && !ctx.core.player(src).hand.is_empty()
                    {
                        let c = ctx.core.player(src).hand[0];
                        ctx.discard_from_hand(src, c);
                    }
                }
            }
        }
    }
    // 伤逝（规则 v260）：体力变化（受伤扣血）后补牌
    shangshix_draw(ctx, d.target);
    // 章武（Boss 刘备，规则 v261）：你受到伤害后（来源存活且非自己）——
    // 你弃置一张牌，来源弃置一张牌，或来源受到 1 点伤害（上游 [1,∞] 自
    // 选+来源 chooseToDiscard/取消双 AI 态度面→双方恒弃首张确定性近似
    // 登记；num=刘备弃牌数可多张登记；弃牌面=手牌首张，两手皆空跳过）
    if let Some(zwdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ZHANGWU)
    {
        if ctx.core.has_skill(d.target, zwdef) && ctx.core.player(d.target).alive {
            if let Some(src) = d.source {
                if src != d.target && ctx.core.player(src).alive {
                    let hold = ctx.core.player(d.target).hand.first().copied();
                    if let Some(c) = hold {
                        ctx.discard_from_hand(d.target, c);
                    }
                    let src_card = ctx.core.player(src).hand.first().copied();
                    if let Some(c) = src_card {
                        ctx.discard_from_hand(src, c);
                    } else {
                        // 来源无牌可弃 → 受 1 点伤害（上游取消分支同口径；
                        // 来源亦空手牌时装备面不弃登记差异）
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(d.target),
                            src,
                            1,
                        )));
                    }
                }
            }
        }
    }
    // 极点（Boss 张角，规则 v263）：你造成伤害后——距离受伤者 1 以内的
    // 一名其他角色判定：黑则其受到 1 点雷伤（上游 source damageAfter
    // chooseTarget get.distance(trigger.player,target)<=1 + judge 黑 →
    // damage("thunder") 同口径；chooseTarget AI→座位序首个确定性近似；
    // 链挡旗标=极点雷伤不自我递归，登记差异：上游 damageAfter 真递归；
    // 判定为内联确定性口径，无改判窗）
    if let Some(jddef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JIDIAN)
    {
        if let Some(src) = d.source {
            if ctx.core.player(src).alive && ctx.core.has_skill(src, jddef) {
                // 链挡：极点造成的雷伤不再触发极点
                if !ctx.core.turn.jidian_chain {
                    let seats = ctx.core.players.len() as PlayerId;
                    if let Some(t) = (0..seats).find(|&t| {
                        t != d.target
                            && ctx.core.player(t).alive
                            && ctx.core.computed_distance(ctx.catalog, d.target, t) <= 1
                    }) {
                        // 置链挡后 emit：嵌套 Damage 帧的极点臂见旗标清除
                        // 并跳过（挡恰一层=极点雷伤不链式，登记差异）
                        ctx.core.turn.jidian_chain = true;
                        let judged = ctx.draw_judge_card();
                        let black = matches!(
                            ctx.catalog.cards[judged as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        );
                        ctx.to_discard(judged);
                        if black {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(src),
                                t,
                                1,
                                crate::event::DamageNature::Thunder,
                                None,
                            )));
                        }
                    }
                } else {
                    // 嵌套极点雷伤到达：清链挡（仅挡一层）
                    ctx.core.turn.jidian_chain = false;
                }
            }
        }
    }
    // 反噬（十殿阎罗转轮王，规则 v269）：每回合第一次受伤后置 terra 旗，
    // 同回合再次受伤 → 随机一名其他角色受 1 点伤害（上游 boss_zlfanshi
    // damageEnd addTempSkill boss_zlfanshi_terra 交替同口径；随机→座位序
    // 首个确定性近似登记；旗清零=回合结束全清，封印解除同位）
    if let Some(fsdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ZLFANSHI)
    {
        if ctx.core.has_skill(d.target, fsdef) && ctx.core.player(d.target).alive {
            if ctx.core.player(d.target).zlfanshi_terra {
                ctx.core.player_mut(d.target).zlfanshi_terra = false;
                let seats = ctx.core.players.len() as PlayerId;
                if let Some(t) = (0..seats).find(|&t| t != d.target && ctx.core.player(t).alive) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(d.target),
                        t,
                        1,
                    )));
                }
            } else {
                ctx.core.player_mut(d.target).zlfanshi_terra = true;
            }
        }
    }
    // 逐厄（Boss 穷奇，规则 v211）：global damageEnd——任何人受到伤害
    // 后（来源存活且非穷奇），穷奇与来源各摸一张（上游 boss_zhue
    // asyncDraw([player, source]) 同口径）
    if let Some(zdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ZHUE)
    {
        // 任一存活穷奇持有者（单 boss 场景恰一）
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == zdef && i.enabled)
            .map(|((o, _), _)| *o)
            .collect();
        for holder in holders {
            if !ctx.core.player(holder).alive {
                continue;
            }
            if let Some(src) = d.source {
                if src != holder && ctx.core.player(src).alive {
                    ctx.draw_cards(holder, 1);
                    ctx.draw_cards(src, 1);
                }
            }
        }
    }
    // 阿鼻（Boss 平等王，规则 v272）：受到有来源的伤害后——来源受 1
    // 点火焰伤害（上游 randomGet fire/thunder→恒定火确定性近似登记；
    // 无来源伤害不触发；新伤害无来源→天然无递归）
    if let Some(abdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ABI)
    {
        if let Some(src) = d.source {
            if ctx.core.has_skill(d.target, abdef)
                && ctx.core.player(d.target).alive
                && ctx.core.player(src).alive
            {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    None,
                    src,
                    1,
                    crate::event::DamageNature::Fire,
                    None,
                )));
            }
        }
    }
    // 远策（Boss 郭嘉，规则 v259）：global damageEnd——其他角色对另一
    // 其他角色造成伤害后（双方存活），判定：黑→受伤角色失 1 体力 / 红→
    // 来源失 1 体力（chooseControl 选项一恒定确定性近似登记——上游 AI
    // 态度选面+rejudge stop 检查+判定修正回调不落；hp>0 门=濒死未决者
    // 跳过登记差异）
    if let Some(ycdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_YUANCE)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == ycdef && i.enabled)
            .map(|((o, _), _)| *o)
            .collect();
        if let Some(holder) = holders.first() {
            if let Some(src) = d.source {
                if ctx.core.player(*holder).alive
                    && src != d.target
                    && ctx.core.player(src).alive
                    && ctx.core.player(d.target).alive
                    && ctx.core.player(src).hp > 0
                    && ctx.core.player(d.target).hp > 0
                {
                    let judged = ctx.draw_judge_card();
                    let black = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    ctx.to_discard(judged);
                    let loser = if black { d.target } else { src };
                    ctx.emit(EventData::LoseHp {
                        player: loser,
                        amount: 1,
                    });
                }
            }
        }
    }
    // 灵渠（Boss 玄武，规则 v237）：每次受到伤害后摸 1 张牌且受击计数
    // +1（上游 boss_lingqu damageEnd draw+storage++ 同口径；0 点伤害照常；
    // 计数=PlayerState.lingqu_count 跨回合持久，手牌上限 +X）
    if let Some(ldef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_LINGQU)
    {
        if ctx.core.has_skill(d.target, ldef) && ctx.core.player(d.target).alive {
            ctx.draw_cards(d.target, 1);
            let p = ctx.core.player_mut(d.target);
            p.lingqu_count = p.lingqu_count.saturating_add(1);
        }
    }
    // 急叮（Boss 明星，规则 v245）：global damageEnd——友方（同侧非己）
    // 受敌方来源伤害后，对来源用虚拟雷杀（LeiSha kind 载体——机器直发不
    // 经 PlayCard 校验；蓐收加成尾面登记差异未建模）
    if let Some(jddef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JIDING)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == jddef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| {
                ctx.core.player(*o).alive
                    && *o != d.target
                    && ctx.core.is_boss_side(*o) == ctx.core.is_boss_side(d.target)
            })
            .collect();
        if let Some(holder) = holders.first() {
            if let Some(src) = d.source {
                if src != *holder
                    && ctx.core.player(src).alive
                    && ctx.core.is_boss_side(src) != ctx.core.is_boss_side(*holder)
                {
                    ctx.use_virtual_card(*holder, CardKind::LeiSha, vec![src]);
                }
            }
        }
    }
    // 忍性（年兽形态，规则 v249）：global damageEnd——持有者非当前回
    // 合角色时，任意角色受到伤害后其摸 1（上游 boss_renxing damageEnd
    // global `_status.currentPhase != player` 同口径）
    if let Some(rxdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_RENXING)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == rxdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current != *o)
            .collect();
        for holder in holders {
            ctx.draw_cards(holder, 1);
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
                // 中毒标记技+蛇毒结算触发技一并授（上游 temp boss_zhongdu
                // +global boss_shedu 组合同批生效）
                for skdef in [
                    zd,
                    ctx.catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_SHEDU)
                        .expect("蛇毒 def"),
                ] {
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
    // 毒妒（Boss 相柳，规则 v223）：你受到伤害时来源获 1 蛇毒标记（自
    // 身不获；上游 boss_duqu damageEnd storage++ 同口径）
    if let Some(dddef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_DUQU)
    {
        if ctx.core.has_skill(d.target, dddef) {
            if let Some(src) = d.source {
                if src != d.target && ctx.core.player(src).alive {
                    let p = ctx.core.player_mut(src);
                    p.shedu = p.shedu.saturating_add(1);
                }
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
    // 冥顽锁定集（Boss 梼杌，规则 v219）：回合内自己使用牌对其他角色
    // 造成伤害后，受伤者入锁定集（首次同时记自己——上游 storage
    // boss_minwan=[player]+add(target) 同口径；回合更替自然清零）
    if let Some(mdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_MINWAN)
    {
        if let Some(src) = d.source {
            if src != d.target
                && ctx.core.turn.current == src
                && ctx.core.has_skill(src, mdef)
                && ctx.core.player(src).alive
            {
                let turn = &mut ctx.core.turn;
                if turn.minwan_targets.is_empty() {
                    turn.minwan_targets.push(src);
                }
                if !turn.minwan_targets.contains(&d.target) {
                    turn.minwan_targets.push(d.target);
                }
            }
        }
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
                ctx.draw_cards(player, 4 - n);
            }
        }
    }
}

fn finish_damage(ctx: &mut Ctx<'_>, root: EventId, mut d: crate::event::DamageData, outcome: crate::event::DamageOutcome) {
    d.settlement.outcome = outcome;
    d.settlement.progress = if outcome == crate::event::DamageOutcome::Zero { crate::event::DamageProgress::ZeroPending } else { crate::event::DamageProgress::Stopped };
    ctx.res.frame_mut(root).expect("damage root").data = EventData::Damage(d);
}
