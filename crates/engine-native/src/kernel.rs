//! 内核固有机制：帧提交动作与内核流程机器。
//!
//! 原则：移动牌、扣血、摸牌、事件推进、濒死/死亡/出牌结算等**规则机制**
//! 归内核；具体技能的条件与流程归内容模块（`content`），两者通过同一套
//! 事件帧 + 机器接口协作。

use crate::catalog::{CardKind, SkillMachine};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{EventData, Phase};
use crate::ids::EventId;
use crate::ids::PlayerId;
use crate::machine::{damage_data, damage_data_with, Ctx, Flow, LogEvent};

/// 帧提交阶段的一次性内核动作。
pub fn commit_frame(ctx: &mut Ctx<'_>, frame_id: EventId) {
    let (kind, data) = {
        let f = ctx.res.frame(frame_id).expect("commit frame");
        (f.kind, f.data.clone())
    };
    match data {
        EventData::Damage(d) => commit_damage(ctx, frame_id, d),
        EventData::LoseHp { player, amount } => commit_lose_hp(ctx, player, amount),
        EventData::CardUsed(u) => commit_card_used(ctx, u),
        EventData::Equipped { player, card, .. } => commit_equip(ctx, frame_id, player, card),
        EventData::Linked { player, .. } => commit_linked(ctx, frame_id, player),
        EventData::Phase { player, phase, begin } => commit_phase(ctx, player, phase, begin),
        EventData::TurnEnded { player } => commit_turn_ended(ctx, player),
        EventData::TurnOverBefore { player, cancelled } => {
            commit_turn_over_before(ctx, player, cancelled)
        }
        EventData::CardsDiscarded { player, .. } => commit_cards_discarded(ctx, player),
        EventData::CardsMoved { to, .. } => commit_cards_moved(ctx, to),
        EventData::Recover { player, .. } => {
            commit_recover_skills(ctx, player)
        }
        // CardsDrawn/Recover/CardsMoved 等在构造时已由助手完成区移动作。
        _ => {
            let _ = kind;
        }
    }
}

/// 伤害提交：读最终帧载荷（转移/防止已生效），扣血并内联濒死检查。
fn commit_damage(ctx: &mut Ctx<'_>, _frame_id: EventId, d: crate::event::DamageData) {
    if d.prevented {
        ctx.log(LogEvent::DamagePrevented { target: d.target });
        return;
    }
    if d.amount == 0 {
        return;
    }
    // 绝情（Boss 张春华，规则 v260；一将包旧绝情）：锁定——你造成的伤害
    // 改为目标失去等量体力（上游 source damageBefore cancel+loseHp(num)
    // 同口径；num=增幅前原始值=d.amount；取消后伤害事件不再结算=受击/
    // 加成类不触发；置于急冻前——上游 damageBefore 同层内容序）
    if let Some(jqdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::JUE_QING)
    {
        if let Some(src) = d.source {
            if src != d.target
                && ctx.core.player(src).alive
                && ctx.core.has_skill(src, jqdef)
            {
                ctx.emit(EventData::LoseHp {
                    player: d.target,
                    amount: d.amount,
                });
                return;
            }
        }
    }
    // 无心（Boss 张春华，规则 v260）伤害半：锁定——你受到的伤害改为失去
    // 1 点体力（上游 damageBefore priority 10 cancel+loseHp() 无参=1 同
    // 口径；置于绝情后=同层 priority 序）
    if let Some(wxdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_WUXIN)
    {
        if ctx.core.has_skill(d.target, wxdef) && ctx.core.player(d.target).alive {
            ctx.emit(EventData::LoseHp {
                player: d.target,
                amount: 1,
            });
            return;
        }
    }
    // 急冻冰封（U05 ow 美，规则 v127）：锁定——冰封目标受所有伤害无
    // 效（上游 jidong2 damageBefore trigger.cancel() 同口径；铁索传导
    // 的独立事件同门控）。置于各伤害增幅钩前：伤害已整体取消，
    // damageBegin 类增幅同上游一样不再结算。
    if ctx.core.player(d.target).jidong_active {
        ctx.log(LogEvent::DamagePrevented { target: d.target });
        return;
    }
    // 诚信免伤（U05 xianjian 明绣，规则 v164）：锁定——免伤状态目标受
    // 所有伤害无效（上游 chengxin2 damageBefore trigger.cancel 同口径）
    if ctx.core.player(d.target).chengxin_active {
        ctx.log(LogEvent::DamagePrevented { target: d.target });
        return;
    }
    // 招祸（Boss 毕方，规则 v227）：锁定——你免疫所有火属性伤害（上游
    // boss_zhaohuo damageBegin4 trigger.cancel 的火伤分支同口径；诚信免
    // 伤同位）
    if d.nature == crate::event::DamageNature::Fire {
        if let Some(zdef) = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_ZHAOHUO)
        {
            if ctx.core.has_skill(d.target, zdef) {
                ctx.log(LogEvent::DamagePrevented { target: d.target });
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
            .skill_by_name(crate::content::standard::BOSS_HUOSHEN)
        {
            if ctx.core.player(d.target).alive
                && ctx.core.has_skill(d.target, hsdef)
            {
                ctx.log(LogEvent::DamagePrevented { target: d.target });
                ctx.recover(d.target, 1, None);
                return;
            }
        }
    }
    // 拟态（Boss 梼杌，规则 v217）：锁定——防止你于回合内受到的伤害
    //（上游 boss_nitai_in damageBegin4 trigger.cancel 同口径：当前回合
    // =持有者）
    if let Some(ndef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_NITAI)
    {
        if ctx.core.has_skill(d.target, ndef)
            && ctx.core.turn.current == d.target
        {
            ctx.log(LogEvent::DamagePrevented { target: d.target });
            return;
        }
    }
    let mut amount = d.amount;
    // 拟态（Boss 梼杌，规则 v217）：锁定——你于回合外受到的火属性伤害
    // +1（上游 boss_nitai_out damageBegin1 num++ 同口径）
    if d.nature == crate::event::DamageNature::Fire {
        if let Some(ndef) = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_NITAI)
        {
            if ctx.core.has_skill(d.target, ndef)
                && ctx.core.turn.current != d.target
            {
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
                .skill_by_name(crate::content::host::WUDIAN_FEIREN_KEY)
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
                    .skill_by_name(crate::content::host::WUDIAN_CIQIU_KEY)
                {
                    if ctx.core.has_skill(src, def) {
                        amount += 1;
                        if amount >= ctx.core.player(d.target).hp as u32 {
                            ctx.core
                                .turn
                                .ciqiu_dying
                                .push(d.target);
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
                .skill_by_name(crate::content::host::WUDIAN_DUWEN_KEY)
            {
                if ctx.core.has_skill(src, def) {
                    amount += 1;
                }
            }
        }
    }
    // 炮台降级（U05 ow，规则 v123）：炮台持有者受伤 1 点降 1 级（上游
    // paotai3 damageEnd forced：storage -= trigger.num；下限 0）
    if ctx.core.player(d.target).paotai_lv > 0 && d.amount > 0 && !d.prevented {
        let lv = ctx.core.player(d.target).paotai_lv;
        ctx.core.player_mut(d.target).paotai_lv = lv.saturating_sub(d.amount as u8);
    }
    // 护甲（G1 扩展）：1:1 抵扣伤害，先于体力扣减
    let hujia = ctx.core.player(d.target).hujia;
    if hujia > 0 {
        let absorbed = (hujia as u32).min(amount);
        ctx.core.player_mut(d.target).hujia = hujia - absorbed as u8;
        amount -= absorbed;
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
        .skill_by_name(crate::content::host::WUDIAN_JIJIA_KEY)
        .map(|def| ctx.core.has_skill(d.target, def))
        .unwrap_or(false);
    if has_jijia {
        let j = ctx.core.player(d.target).jijia_hp;
        if j > 0 {
            let absorbed = (j as u32).min(amount);
            ctx.core.player_mut(d.target).jijia_hp = j - absorbed as u8;
            amount -= absorbed;
            ctx.log(LogEvent::HujiaAbsorbed {
                player: d.target,
                amount: absorbed,
            });
        }
    }
    if amount == 0 {
        return;
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
            .skill_by_name(crate::content::standard::BOSS_ZUIJIU)
        {
            if let Some(src) = d.source {
                if ctx.core.has_skill(src, zjdef) {
                    amount += 1;
                }
            }
        }
    }
    // 灵渠（Boss 玄武，规则 v237）：受到的大于 1 点伤害视为 0 点（上游
    // boss_lingqu cancel damageBegin4 num>1 → num=0 同口径；置于增幅后
    // 统计前——终值>1 才归零；0 点伤害仍完成事件→受击摸 1/计数照常）
    if amount > 1 {
        if let Some(ldef) = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_LINGQU)
        {
            if ctx.core.has_skill(d.target, ldef) {
                amount = 0;
            }
        }
    }
    // damageEnd num 同口径）
    if let Some(src) = d.source {
        if src != d.target {
            let p = ctx.core.player_mut(src);
            p.turn_damage = p.turn_damage.saturating_add(amount.min(255) as u8);
        }
    }
    let p = ctx.core.player_mut(d.target);
    p.hp -= amount as i32;
    ctx.log(LogEvent::DamageDealt {
        source: d.source,
        target: d.target,
        amount: d.amount,
        redirected_from: d.redirect_chain.clone(),
        card: d.card,
    });
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
            ctx.recover(src, d.amount, None);
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
                && ctx.core.player(src).equips.iter().any(|c| {
                    ctx.catalog.card_kind(*c) == CardKind::Qinglonglingzhu
                })
            {
                let he_len = ctx.core.player(t).hand.len()
                    + ctx.core.player(t).equips.len();
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
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            t,
                            c,
                        );
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
        .skill_by_name(crate::content::standard::BOSS_YINZEI)
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
        .skill_by_name(crate::content::standard::BOSS_ZHANGWU)
    {
        if ctx.core.has_skill(d.target, zwdef)
            && ctx.core.player(d.target).alive
        {
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
        .skill_by_name(crate::content::standard::BOSS_JIDIAN)
    {
        if let Some(src) = d.source {
            if ctx.core.player(src).alive && ctx.core.has_skill(src, jddef) {
                // 链挡：极点造成的雷伤不再触发极点
                if !ctx.core.turn.jidian_chain {
                    let seats = ctx.core.players.len() as PlayerId;
                    if let Some(t) = (0..seats).find(|&t| {
                        t != d.target
                            && ctx.core.player(t).alive
                            && ctx
                                .core
                                .computed_distance(ctx.catalog, d.target, t)
                                <= 1
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
                            ctx.emit(EventData::Damage(
                                crate::machine::damage_data_with(
                                    Some(src),
                                    t,
                                    1,
                                    crate::event::DamageNature::Thunder,
                                    None,
                                ),
                            ));
                        }
                    }
                } else {
                    // 嵌套极点雷伤到达：清链挡（仅挡一层）
                    ctx.core.turn.jidian_chain = false;
                }
            }
        }
    }
    // 逐厄（Boss 穷奇，规则 v211）：global damageEnd——任何人受到伤害
    // 后（来源存活且非穷奇），穷奇与来源各摸一张（上游 boss_zhue
    // asyncDraw([player, source]) 同口径）
    if let Some(zdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_ZHUE)
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
    // 远策（Boss 郭嘉，规则 v259）：global damageEnd——其他角色对另一
    // 其他角色造成伤害后（双方存活），判定：黑→受伤角色失 1 体力 / 红→
    // 来源失 1 体力（chooseControl 选项一恒定确定性近似登记——上游 AI
    // 态度选面+rejudge stop 检查+判定修正回调不落；hp>0 门=濒死未决者
    // 跳过登记差异）
    if let Some(ycdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_YUANCE)
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
                    ctx.emit(EventData::LoseHp { player: loser, amount: 1 });
                }
            }
        }
    }
    // 灵渠（Boss 玄武，规则 v237）：每次受到伤害后摸 1 张牌且受击计数
    // +1（上游 boss_lingqu damageEnd draw+storage++ 同口径；0 点伤害照常；
    // 计数=PlayerState.lingqu_count 跨回合持久，手牌上限 +X）
    if let Some(ldef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_LINGQU)
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
        .skill_by_name(crate::content::standard::BOSS_JIDING)
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
        .skill_by_name(crate::content::standard::BOSS_RENXING)
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
        .skill_by_name(crate::content::standard::BOSS_YANDU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, dudef)
            && !ctx.core.has_skill(holder, dudef)
        {
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
        .skill_by_name(crate::content::standard::BOSS_JICAI)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, jdef)
            && !ctx.core.has_skill(holder, jdef)
        {
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
        .skill_by_name(crate::content::standard::BOSS_CUIDUI)
    {
        let zdef = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_ZHONGDU);
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
                        .skill_by_name(crate::content::standard::BOSS_SHEDU)
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
        .skill_by_name(crate::content::standard::BOSS_DUQU)
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
        .skill_by_name(crate::content::standard::BOSS_ECHOU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, edef)
            && !ctx.core.has_skill(holder, edef)
        {
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
        .skill_by_name(crate::content::standard::BOSS_LUANCHANG)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, ldef)
            && !ctx.core.has_skill(holder, ldef)
        {
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
        .skill_by_name(crate::content::standard::BOSS_SIPU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, sdef)
            && !ctx.core.has_skill(holder, sdef)
        {
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
        .skill_by_name(crate::content::standard::BOSS_XUSHI)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, xdef)
            && !ctx.core.has_skill(holder, xdef)
        {
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
        .skill_by_name(crate::content::standard::BOSS_YANYU)
    {
        let holder = d.target;
        if ctx.core.has_skill_any_state(holder, ydef)
            && !ctx.core.has_skill(holder, ydef)
        {
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
        .skill_by_name(crate::content::standard::BOSS_MINWAN)
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
    // 内联濒死检查：体力 ≤ 0 且尚未处于濒死。
    let entering = ctx.core.player(d.target).hp <= 0 && !ctx.core.player(d.target).dying;
    if entering {
        ctx.core.player_mut(d.target).dying = true;
        ctx.log(LogEvent::DyingEntered { player: d.target });
        ctx.emit(EventData::Dying {
            player: d.target,
            source: d.source,
        });
    }
    // 铁索连环传导：属性伤害结算后，传导给其余被链存活玩家（座位顺序），
    // 随后全部解除连环。被防止/零点伤害不传导；传导伤害是完全的伤害结算
    // （可被转移/响应/再触发濒死），且因链已先解除而不再级联——语义取舍
    // 见覆盖清单行为差异表。
    if d.nature != crate::event::DamageNature::Normal {
        let target = d.target;
        let chained: Vec<crate::ids::PlayerId> = ctx
            .core
            .seat_order_from(target)
            .into_iter()
            .filter(|p| *p != target && ctx.core.player(*p).linked)
            .collect();
        if !chained.is_empty() {
            // 先解除全部连环（含目标自身）
            for p in ctx
                .core
                .alive_players()
                .filter(|p| ctx.core.player(*p).linked)
                .collect::<Vec<_>>()
            {
                ctx.core.player_mut(p).linked = false;
                ctx.log(LogEvent::Linked {
                    player: p,
                    linked: false,
                });
            }
            for p in chained {
                ctx.emit(EventData::Damage(crate::event::DamageData {
                    source: d.source,
                    target: p,
                    amount: d.amount,
                    nature: d.nature,
                    prevented: false,
                    redirect_chain: vec![],
                    card: d.card,
                    from_chain: true,
                }));
            }
        }
    }
}

/// 失去体力提交（逐技对照批三 v77）：直接扣减体力——不经护甲抵扣、
/// 无伤害前置窗口/铁索传导/伤害来源触发技（LoseHp 是独立事件种）；
/// 体力 ≤0 与伤害同口径进入濒死（上游 loseHp 语义）。
pub(crate) fn commit_lose_hp(ctx: &mut Ctx<'_>, player: PlayerId, amount: u32) {
    if amount == 0 {
        return;
    }
    let p = ctx.core.player_mut(player);
    p.hp -= amount as i32;
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

/// 枳枉（十殿阎罗五官王，规则 v265）：其他角色获得牌结算后（CardsMoved
/// 面——摸牌阶段摸牌走 CardsDrawn 天然隔离=上游排除 draw 同口径），其
/// 随机弃置一张手牌（随机→首张确定性近似登记；planetarian 彩蛋分支不
/// 落登记）
fn commit_cards_moved(ctx: &mut Ctx<'_>, to: PlayerId) {
    if let Some(zwdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_ZHIWANG)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == zwdef && i.enabled)
            .map(|((o, _), _)| *o)
            .collect();
        if let Some(holder) = holders.first() {
            if to != *holder && ctx.core.player(to).alive {
                let pick = ctx.core.player(to).hand.first().copied();
                if let Some(c) = pick {
                    ctx.discard_from_hand(to, c);
                }
            }
        }
    }
}

/// 伤逝（Boss 张春华，规则 v260）补牌半：
fn shangshix_draw(ctx: &mut Ctx<'_>, player: PlayerId) {
    if let Some(ssdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_SHANGSHIX)
    {
        if ctx.core.player(player).alive && ctx.core.has_skill(player, ssdef) {
            let n = ctx.core.player(player).hand.len();
            if n < 4 {
                ctx.draw_cards(player, 4 - n);
            }
        }
    }
}

/// 连环切换提交：翻转横置状态并写回帧载荷。
fn commit_linked(ctx: &mut Ctx<'_>, frame_id: EventId, player: PlayerId) {
    let next = !ctx.core.player(player).linked;
    ctx.core.player_mut(player).linked = next;
    if let Some(f) = ctx.res.frame_mut(frame_id) {
        if let EventData::Linked { linked, .. } = &mut f.data {
            *linked = next;
        }
    }
    ctx.log(LogEvent::Linked {
        player,
        linked: next,
    });
}

/// 出牌提交：牌离开手牌区。杀置桌面等待结算；桃立即回复。
/// 猴子偷桃（E03）取消的使用的牌已在确认机器中移走，跳过结算。
fn commit_card_used(ctx: &mut Ctx<'_>, u: crate::event::CardUseData) {
    // 奇佐副本载体槽（Boss 郭嘉，规则 v259）：任一非载体卡牌提交即清
    //（副本用牌自身的提交保留槽位，供其 AfterTriggers 过滤拒递归）
    if ctx.core.turn.qizuo_copy != Some(u.card) {
        ctx.core.turn.qizuo_copy = None;
    }
    if u.cancelled {
        // 取消痕迹已在猴子机器的 CardsGiven/CardsDiscarded 中观测。
        return;
    }
    ctx.take_hand_card(u.player, u.card);
    // 全局用牌计数（U05 xianjian 小蛮，规则 v163 灵泉门控；上游
    // getHistory("useCard") 同口径——未被取消的使用全计）
    ctx.core.player_mut(u.player).total_used += 1;
    // 邪酬（Boss 相柳，规则 v223）：激活态下其他角色使用【桃】/【酒】
    // → 相柳获 1 蛇毒标记（上游 boss_echou global useCard 同口径）
    let effective_kind = ctx.catalog.card_kind(u.card);
    if matches!(
        u.use_as.unwrap_or(effective_kind),
        CardKind::Tao | CardKind::Jiu
    ) {
        if let Some(edef) = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_ECHOU)
        {
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == edef && i.enabled)
                .map(|((o, _), _)| *o)
                .collect();
            for holder in holders {
                if holder != u.player && ctx.core.player(holder).alive {
                    let p = ctx.core.player_mut(holder);
                    p.shedu = p.shedu.saturating_add(1);
                }
            }
        }
    }
    // 冥顽（Boss 梼杌，规则 v219）：回合内锁定集已激活（非空）时每用
    // 一张牌摸一张（上游 boss_minwan_draw useCard 同口径）
    if ctx.core.turn.current == u.player && !ctx.core.turn.minwan_targets.is_empty() {
        if let Some(mdef) = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_MINWAN)
        {
            if ctx.core.has_skill(u.player, mdef) {
                ctx.draw_cards(u.player, 1);
            }
        }
    }
    // 娇孝（Boss 甄姬，规则 v262）：成为红色牌的目标后摸 1（上游 target
    // useCardToTargeted frequent 同口径；红色=实体花色 Heart/Diamond；
    // 含延时自置目标）
    if let Some(jxdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_JIAOXIA)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == jxdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive)
            .collect();
        if let Some(holder) = holders.first() {
            let red = matches!(
                ctx.catalog.cards[u.card as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            // 显式目标面：targets 含持有者；闪电自置：targets 空（隐式
            // 自目标，上游 useCardToTargeted target=使用者同口径）
            let is_self_shandian = u.targets.is_empty()
                && u.player == *holder
                && effective_kind == CardKind::ShanDian;
            if red && (u.targets.contains(holder) || is_self_shandian) {
                ctx.draw_cards(*holder, 1);
            }
        }
    }
    ctx.log(LogEvent::CardPlayed {
        card: u.card,
        player: u.player,
        targets: u.targets.clone(),
    });
    // view-as 声明持久化：延时锦囊转化（国色）需跨阶段保存声明牌种，
    // 判定生效时按声明牌种执行（目录牌种不再可靠）。
    if let Some(as_kind) = u.use_as {
        ctx.core.declared_kinds.insert(u.card, as_kind);
    }
    // view-as 转化（武圣/龙胆）：以声明牌种结算，而非目录牌种
    let effective = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
    match effective {
        CardKind::Tao => {
            ctx.to_discard(u.card);
            ctx.recover(u.player, 1, Some(u.card));
        }
        CardKind::Zhufangshenshi => {
            // 贴房绅视（U05 swd 第十批，规则 v94）：本回合无视距离指定
            // 该目标（sha 距离校验豁免）并摸一张牌。登记差异：上游为
            // 锦囊可被无懈（AI 权重 0 实战不戒），原生简化为直接结算。
            let t = *u.targets.first().expect("zhufangshenshi has one target");
            ctx.core.turn.zhufangshenshi_target = Some(t);
            ctx.to_discard(u.card);
            ctx.draw_cards(u.player, 1);
        }
        CardKind::Caoyao => {
            // 草药（U05 swd 第三批，规则 v87）：为受伤目标回复 1 点体力
            //（目标合法性在 PlayCard 门控：距离 1 + hp<maxHp）
            ctx.to_discard(u.card);
            let t = *u.targets.first().expect("caoyao has one target");
            ctx.recover(t, 1, Some(u.card));
        }
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
            // 出牌阶段限用一次（普通/火/雷杀共享）；龙颅山峰地图效果
            //（mtg 首切片，规则 v100）下第二张杀走额外标记——若本次为
            // 加成额外杀则置 land_sha_used，否则置常规已用
            if ctx.core.turn.sha_used_play_phase {
                if ctx.core.current_land
                    == Some(crate::catalog::CardKind::Longlushanfeng)
                {
                    ctx.core.turn.land_sha_used = true;
                }
            } else {
                ctx.core.turn.sha_used_play_phase = true;
            }
            // 冰河要塞地图效果（mtg 第八批，规则 v107）：锁定全局——使
            // 用杀时若有牌须弃一张（上游 chooseToDiscard 可取消+玩家选；
            // 原生 RNG 代抽不可取消，登记差异）。当前弃牌阶段的水没计数
            // 不受影响（非弃牌阶段）。
            if ctx.core.current_land == Some(crate::catalog::CardKind::Bingheyaosai) {
                let p = ctx.core.player(u.player);
                let has_cards = !p.hand.is_empty() || !p.equips.is_empty();
                if has_cards {
                    let from_hand = if p.hand.is_empty() {
                        false
                    } else if p.equips.is_empty() {
                        true
                    } else {
                        ctx.core.rng.gen_range(2u32) == 0
                    };
                    if from_hand {
                        let n = ctx.core.player(u.player).hand.len() as u32;
                        let j = ctx.core.rng.gen_range(n) as usize;
                        let c = ctx.core.player_mut(u.player).hand.remove(j);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: u.player,
                            cards: vec![c],
                        });
                    } else {
                        let n = ctx.core.player(u.player).equips.len() as u32;
                        let j = ctx.core.rng.gen_range(n) as usize;
                        let c = ctx.core.player_mut(u.player).equips.remove(j);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            u.player,
                            c,
                        );
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: u.player,
                            cards: vec![c],
                        });
                    }
                }
            }
            ctx.core.table.push(u.card);
        }
        CardKind::Jiu | CardKind::Tianxianjiu => {
            // 酒/天仙酒：命令边界已校验每回合一次——置本回合已用 + 待
            // 消耗效果（天仙酒为酒的 swd 变体，规则 v89；登记差异：与酒
            // 共享每回合一次计数）
            ctx.to_discard(u.card);
            ctx.core.turn.jiu_used = true;
            ctx.core.turn.jiu_active = true;
        }
        // Le/Bing/ShanDian：先置桌面，经无懈窗口（使用时开窗，对齐旧线）
        // 由 PlaceDelayedMachine 放入判定区或弃置。
        _ => {
            ctx.core.table.push(u.card);
        }
    }
    // 突击计数（U07 ow，规则 v118）：出牌阶段使用任意牌 +1（取消不计；
    // 等值牌种分支已各自置 table，此处只累计）
    ctx.core.turn.cards_used_play_phase += 1;
}

/// 装备提交：牌入手牌区到对应装备栏，同栏旧牌替换入弃牌堆；
/// 替换结果写回帧载荷（哈希可见）。
/// G1 装备技能生命周期：装备 `card` 时授予其声明的目录技能。
pub(crate) fn grant_equip_skills(ctx: &mut Ctx<'_>, player: PlayerId, card: crate::ids::CardId) {
    let names: Vec<&'static str> = ctx.catalog.cards[card as usize].equip_skills.clone();
    for name in names {
        if let Some(def) = ctx.catalog.skill_by_name(name) {
            ctx.core.skills.entry((player, def)).or_insert(crate::state::SkillInstance {
                owner: player,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            });
        }
    }
}

/// G1 装备技能生命周期：装备 `card` 离场时撤销其技能——若该技能不同时
/// 由玩家剩余的其他装备提供（跨槽同名技能共存）。
pub(crate) fn revoke_equip_skills(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    let names: Vec<&'static str> = catalog.cards[card as usize].equip_skills.clone();
    for name in names {
        let Some(def) = catalog.skill_by_name(name) else { continue };
        let still_provided = core
            .player(player)
            .equips
            .iter()
            .any(|c| catalog.cards[*c as usize].equip_skills.contains(&name));
        if !still_provided {
            core.skills.remove(&(player, def));
        }
    }
    // G1 索链甲：离场装备声明横置且玩家无其他同类装备 → 重置横置
    if catalog.cards[card as usize].link_on_equip {
        let still = core
            .player(player)
            .equips
            .iter()
            .any(|c| catalog.cards[*c as usize].link_on_equip);
        if !still {
            core.player_mut(player).linked = false;
        }
    }
}

fn commit_equip(ctx: &mut Ctx<'_>, frame_id: EventId, player: PlayerId, card: crate::ids::CardId) {
    let kind = ctx.catalog.card_kind(card);
    let Some(_slot) = kind.equip_slot() else {
        panic!("commit_equip on non-equip card");
    };
    ctx.take_hand_card(player, card);
    let replaced = ctx
        .core
        .player(player)
        .equips
        .iter()
        .copied()
        .find(|c| ctx.catalog.card_kind(*c) == kind);
    if let Some(old) = replaced {
        ctx.core.player_mut(player).equips.retain(|c| *c != old);
        revoke_equip_skills(ctx.core, ctx.catalog, player, old);
        ctx.to_discard(old);
    }
    ctx.core.player_mut(player).equips.push(card);
    grant_equip_skills(ctx, player, card);
    // G1 索链甲：装备时横置
    if ctx.catalog.cards[card as usize].link_on_equip {
        let was = ctx.core.player(player).linked;
        ctx.core.player_mut(player).linked = true;
        if !was {
            ctx.log(LogEvent::Linked { player, linked: true });
        }
    }
    if let Some(f) = ctx.res.frame_mut(frame_id) {
        if let EventData::Equipped { replaced: r, .. } = &mut f.data {
            *r = replaced;
        }
    }
    ctx.log(LogEvent::Equipped {
        player,
        card,
        replaced,
    });
}

/// 阶段帧提交：PhaseEnd 推进阶段或转入回合收束；PhaseBegin 执行入场效果。
fn commit_phase(ctx: &mut Ctx<'_>, player: PlayerId, phase: Phase, begin: bool) {
    // 翻面角色回合跳过（E03 翻面 + U05 ow 美，规则 v127）：翻面者回合
    // 的首个阶段帧改为「先经翻面前置取消窗口翻回（上游 _turnover
    // phaseBefore 强制取消 + turnOver 翻回同口径），再直达回合收束」
    // ——整回合无任何阶段帧（摸牌/出牌/弃牌全跳过），座位/轮次计数
    // 不变（回合槽照常消耗）。
    if begin
        && phase == Phase::Judge
        && ctx.core.player(player).face_down
    {
        // 紧急冷冻解除（U05 swd 灵剑，规则 v198）：翻回即解（上游
        // addTempSkill {player:'turnOverAfter'} 同口径）
        ctx.core.player_mut(player).jileng_active = false;
        ctx.emit(EventData::TurnOverBefore { player, cancelled: false });
        // 混乱解除（规则 v259）：整回合跳过路径同口径（回合收束=解除）
        ctx.core.player_mut(player).boss_mad = false;
        ctx.emit(EventData::TurnEnded { player });
        return;
    }
    // 强固解除（U05 ow 奥丽莎，规则 v136）：自己回合判定阶段开始——
    // 上游 addTempSkill("qianggu2") 至 phaseBegin 同口径
    if begin && phase == Phase::Judge && ctx.core.player(player).qianggu_active {
        ctx.core.player_mut(player).qianggu_active = false;
    }
    // 潜行解除（U05 ow 黑影，规则 v139）：自己回合判定阶段开始——上游
    // tempHide 至 phaseBeginStart 同口径
    if begin && phase == Phase::Judge && ctx.core.player(player).yinshen_active {
        ctx.core.player_mut(player).yinshen_active = false;
    }
    // 嘲讽解除（U05 swd 灵剑圣休号角，规则 v198）：自己回合判定阶段开始
    // ——上游 addTempSkill {player:'phaseBegin'} 同口径
    if begin && phase == Phase::Judge && ctx.core.player(player).chaofeng_active {
        ctx.core.player_mut(player).chaofeng_active = false;
    }
    // 星体改造重置（U05 swd 灵剑，规则 v198）：自己回合判定阶段开始——
    // 上游 phaseEnd 移除+清零在「其下一结束阶段」生效；原生以本回合开始
    // 重置近似（差一夜，登记差异：重置时点=下一回合开始而非上一结束）
    if begin && phase == Phase::Judge && ctx.core.player(player).xingtigaizao_count > 0 {
        ctx.core.player_mut(player).xingtigaizao_count = 0;
    }
    // 蛊舞递减（U05 xianjian 阿奴，规则 v154）：自己每阶段结束 -1（上游
    // anwugu2 phaseAfter 递减同口径）；归 0 后移除（出牌受限/手牌上限随
    // 之解除）。
    if !begin && ctx.core.player(player).anwugu_count > 0 {
        ctx.core.player_mut(player).anwugu_count -= 1;
    }
    // 月行（U05 xianjian 龙幽，规则 v159）：出牌阶段结束解除距离基数交
    // 换（双向互指一并清除；上游 yuexing2 临时技 phaseUseAfter 同口径）
    if !begin && phase == Phase::Play {
        if ctx.core.player(player).yuexing_pair.is_some() {
            if let Some(pair) = ctx.core.player_mut(player).yuexing_pair.take() {
                if ctx.core.player(pair).yuexing_pair == Some(player) {
                    ctx.core.player_mut(pair).yuexing_pair = None;
                }
            }
        }
    }
    // 逐月（U05 xianjian 云天河，规则 v161）：出牌阶段结束清白名单
    //（上游 zhuyue2 phaseUseEnd silent 同口径）
    if !begin && phase == Phase::Play {
        ctx.core.turn.zhuyue_targets.clear();
    }
    // 诚信免伤清除（U05 xianjian 明绣，规则 v164）：本人 End 阶段结束
    //（登记差异：上游 chengxin2 存活至最近阶段结束）
    if !begin && phase == Phase::End && ctx.core.player(player).chengxin_active {
        ctx.core.player_mut(player).chengxin_active = false;
    }
    // 屏障标志转计数（U05 ow 查莉娅，规则 v138）：自己回合判定阶段开
    // 始——上回合两标志各计一次并清除（上游 pingzhang_count phaseBegin
    // silent 同口径；应用技在随后摸牌阶段读计数）
    if begin && phase == Phase::Judge {
        let used = u8::from(ctx.core.player(player).pingzhang_self_used)
            + u8::from(ctx.core.player(player).pingzhang_other_used);
        let p = ctx.core.player_mut(player);
        p.pingzhang_count = used;
        p.pingzhang_self_used = false;
        p.pingzhang_other_used = false;
    }
    ctx.log(LogEvent::PhaseChanged {
        player,
        phase,
        begin,
    });
    // 全局阶段计数（U07 ow 自愈，规则 v120）：每阶段开始 +1（上游
    // game.phaseNumber 同口径）
    if begin {
        ctx.core.turn.phase_number += 1;
    }
    if begin {
        match phase {
            Phase::Judge => {
                if !ctx.core.turn.turn_opened {
                    ctx.reset_turn_limits(player);
                    ctx.core.turn.turn_opened = true;
                    // 回合伤害统计清零（U05 xianjian 明绣，规则 v160 灵火
                    // 门控；上游 getStat 每回合重置同口径）——在本人回合
                    // 开始清（PhaseEnd 触发晚于收束提交，见规则基准）
                    ctx.core.player_mut(player).turn_damage = 0;
                    // 忘忧戳清零（U05 xianjian 紫萱，规则 v162）：存活至
                    // 标记者自己下回合开始（上游 addTempSkill 当前回合结
                    // 束——登记差异：保留期延长）
                    ctx.core.player_mut(player).wangyou3_round = 0;
                    // 摄灵回合次数清零（U05 xianjian 南宫煌，规则 v168）
                    ctx.core.turn.sheling_used = 0;
                    // 射影回合清零（U05 xianjian 苏媚，规则 v175）
                    ctx.core.turn.sheying_used = false;
                    // 张目回合清零（U05 xianjian 南宫煌，规则 v181）
                    ctx.core.turn.zhangmu_used = false;
                    // 红莲（Boss 毕方，规则 v228）：回合开始——顺序获牌
                    // 堆前缀红牌至多 3 张（上游随机 0-3 确定性近似），并对
                    // 座位序前至多 3 名其他角色各造成 1 点火伤（上游随机
                    // 3-0 名确定性近似）
                    if let Some(hdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_HONGLIANX)
                    {
                        if ctx.core.has_skill(player, hdef) {
                            let mut gained = 0usize;
                            while gained < 3 && !ctx.core.deck.is_empty() {
                                let c = ctx.core.deck[ctx.core.deck.len() - 1];
                                if matches!(
                                    ctx.catalog.cards[c as usize].suit,
                                    crate::catalog::Suit::Heart
                                        | crate::catalog::Suit::Diamond
                                ) {
                                    ctx.core.deck.pop();
                                    ctx.core.player_mut(player).hand.push(c);
                                    gained += 1;
                                } else {
                                    break;
                                }
                            }
                            if gained > 0 {
                                ctx.log(LogEvent::CardsDrawn {
                                    player,
                                    cards: ctx.core.player(player)
                                        .hand
                                        .iter()
                                        .rev()
                                        .take(gained)
                                        .copied()
                                        .collect::<Vec<_>>(),
                                });
                            }
                            let targets: Vec<PlayerId> = ctx
                                .core
                                .seat_order_from(player)
                                .into_iter()
                                .filter(|t| *t != player && ctx.core.player(*t).alive)
                                .take(3)
                                .collect();
                            for t in targets {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(player),
                                    t,
                                    1,
                                    crate::event::DamageNature::Fire,
                                    None,
                                )));
                            }
                        }
                    }
                    // 绝红（Boss 共工，规则 v244）：准备阶段——敌方全
                    // 体逐个：有牌者，有装备 → 装备全弃（撤技能+入弃堆）；
                    // 无装备 → 随机弃一张手牌（随机→手牌首张确定性近似
                    // 登记先例族）
                    if let Some(jhdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_JUEHONG)
                    {
                        if ctx.core.has_skill(player, jhdef) {
                            let seats = ctx.core.players.len() as PlayerId;
                            for t in 0..seats {
                                if t == player || !ctx.core.player(t).alive {
                                    continue;
                                }
                                if ctx.core.is_boss_side(t)
                                    == ctx.core.is_boss_side(player)
                                {
                                    continue;
                                }
                                let has_cards = !ctx.core.player(t).hand.is_empty()
                                    || !ctx.core.player(t).equips.is_empty();
                                if !has_cards {
                                    continue;
                                }
                                let equips: Vec<crate::ids::CardId> = ctx
                                    .core
                                    .player(t)
                                    .equips
                                    .iter()
                                    .copied()
                                    .collect();
                                if !equips.is_empty() {
                                    for c in equips {
                                        ctx.core
                                            .player_mut(t)
                                            .equips
                                            .retain(|x| *x != c);
                                        revoke_equip_skills(
                                            ctx.core,
                                            ctx.catalog,
                                            t,
                                            c,
                                        );
                                        ctx.to_discard(c);
                                    }
                                } else if !ctx.core.player(t).hand.is_empty() {
                                    let c = ctx.core.player(t).hand[0];
                                    ctx.discard_from_hand(t, c);
                                }
                            }
                        }
                    }
                    // 夹击（Boss 明星，规则 v245）：准备阶段——相邻（环
                    // 上/下家）敌方中有坐骑（+1/-1 马槽）者 → 坐骑全弃
                    //（撤技能+入弃堆）
                    if let Some(jzdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_JIAZU)
                    {
                        if ctx.core.has_skill(player, jzdef) {
                            for t in [ctx.core.ring_prev(player), ctx.core.ring_next(player)] {
                                if t == player || !ctx.core.player(t).alive {
                                    continue;
                                }
                                if ctx.core.is_boss_side(t)
                                    == ctx.core.is_boss_side(player)
                                {
                                    continue;
                                }
                                let mounts: Vec<crate::ids::CardId> = ctx
                                    .core
                                    .player(t)
                                    .equips
                                    .iter()
                                    .copied()
                                    .filter(|c| {
                                        matches!(
                                            ctx.catalog.card_kind(*c).equip_slot(),
                                            Some(crate::catalog::EquipSlot::HorsePlus)
                                                | Some(crate::catalog::EquipSlot::HorseMinus)
                                        )
                                    })
                                    .collect();
                                for c in mounts {
                                    ctx.core.player_mut(t).equips.retain(|x| *x != c);
                                    revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                    ctx.to_discard(c);
                                }
                            }
                        }
                    }
                    // 造异异变（运行时授予技，规则 v246）：回合开始——
                    // 敌方中体力最低者全体失去全部体力（上游 boss_zaoyi_
                    // hp phaseZhunbeiBegin loseHp(min) 同口径）
                    if let Some(zhdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_ZAOYI_HP)
                    {
                        if ctx.core.has_skill(player, zhdef) {
                            let seats = ctx.core.players.len() as PlayerId;
                            let enemies: Vec<PlayerId> = (0..seats)
                                .filter(|&t| {
                                    ctx.core.player(t).alive
                                        && ctx.core.is_boss_side(t)
                                            != ctx.core.is_boss_side(player)
                                })
                                .collect();
                            if let Some(min) = enemies
                                .iter()
                                .map(|&t| ctx.core.player(t).hp)
                                .min()
                            {
                                if min > 0 {
                                    for t in enemies {
                                        if ctx.core.player(t).hp == min {
                                            ctx.emit(EventData::LoseHp {
                                                player: t,
                                                amount: min as u32,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 形态轮换（Boss 年兽系，规则 v249）：回合开始——
                    // 交替旗标首次置位不换将（初始形态即条目形态），此后
                    // 每回合轮换至另一「形态轮换」携带将（assembly 重挂+
                    // hp 保留——versus 替补换将先例同构；随机序列→确定性
                    // 互斥轮转登记）
                    if let Some(nbdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_NBIANSHEN)
                    {
                        if ctx.core.has_skill(player, nbdef) {
                            let armed = ctx.core.player(player).nbianshen_toggle;
                            if armed {
                                // 启动后每回合轮换（上游 storage 存在后每
                                // 次 phaseBefore 均换同口径）
                                // 轮换：找另一携带「形态轮换」的 BOSS 段将
                                let base = (ctx.catalog.general_count()
                                    - crate::content::standard::BOSS_GENERALS
                                        .len())
                                    as u16;
                                let cur = ctx.core.player(player).general_slots[0];
                                let other = (base
                                    ..ctx.catalog.general_count() as u16)
                                    .find(|&d| {
                                        Some(d) != cur
                                            && ctx
                                                .catalog
                                                .general_meta(d)
                                                .map(|m| {
                                                    m.2.contains(&crate::content::standard::BOSS_NBIANSHEN)
                                                })
                                                .unwrap_or(false)
                                    });
                                if let (Some(other), Some(cur)) = (other, cur) {
                                    let _ = cur;
                                    // 技能实例清空+重挂（区域保留）
                                    let removed: Vec<crate::ids::SkillDefId> =
                                        ctx
                                            .core
                                            .skills
                                            .keys()
                                            .filter(|(o, _)| *o == player)
                                            .map(|(_, d)| *d)
                                            .collect();
                                    for d in removed {
                                        ctx.core.skills.remove(&(player, d));
                                    }
                                    ctx.core.player_mut(player).general_slots[0] =
                                        Some(other);
                                    crate::engine::apply_general_assembly(
                                        ctx.core,
                                        ctx.catalog,
                                        player,
                                        0,
                                    );
                                    // hp/手牌/装备/判定区保留（上游 init 保
                                    // 留 hp/maxHp/hujia 同口径）
                                    ctx.log(LogEvent::SkillToggled {
                                        player,
                                        skill: "形态轮换",
                                        enabled: true,
                                    });
                                }
                            }
                            ctx.core.player_mut(player).nbianshen_toggle = true;
                        }
                    }
                    // 自润（Boss 玄冥，规则 v247）：准备阶段——全体角色
                    // 按座位序各摸牌：有装备者摸 2、无装备者摸 1（上游
                    // boss_zirun phaseZhunbeiBegin asyncDraw 同口径）
                    if let Some(zrdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_ZIRUN)
                    {
                        if ctx.core.has_skill(player, zrdef) {
                            let seats = ctx.core.players.len() as PlayerId;
                            for t in 0..seats {
                                if !ctx.core.player(t).alive {
                                    continue;
                                }
                                let n = if ctx.core.player(t).equips.is_empty() {
                                    1
                                } else {
                                    2
                                };
                                ctx.draw_cards(t, n);
                            }
                        }
                    }
                    // 魔道（Boss 夜叉，规则 v253）：准备阶段摸 2 张牌
                    if let Some(mddef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_MODAO)
                    {
                        if ctx.core.has_skill(player, mddef) {
                            ctx.draw_cards(player, 2);
                        }
                    }
                    // 猛态（Boss 年兽，规则 v248）：准备阶段置双旗标
                    //（上游 boss_mengtai_begin storage 置 true 同口径；
                    // 摸牌/出牌阶段开始时清位）
                    if let Some(mtdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_MENGTAI)
                    {
                        if ctx.core.has_skill(player, mtdef) {
                            ctx.core.turn.mengtai_draw_pending = true;
                            ctx.core.turn.mengtai_use_pending = true;
                        }
                    }
                    // 睿智（年兽睿智形态，规则 v250）：准备阶段——其余
                    // 角色（手牌>1 或装备>1）保留一张手牌和一张装备区内
                    // 的牌，弃置其余（保留哪张由目标自选——原生保留首张
                    // 确定性近似登记）
                    if let Some(rzdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_RUIZHI)
                    {
                        let holders: Vec<PlayerId> = ctx
                            .core
                            .skills
                            .iter()
                            .filter(|((_, dd), i)| *dd == rzdef && i.enabled)
                            .map(|((o, _), _)| *o)
                            .filter(|o| ctx.core.player(*o).alive)
                            .collect();
                        if !holders.is_empty() {
                            let seats = ctx.core.players.len() as PlayerId;
                            for t in 0..seats {
                                if !ctx.core.player(t).alive {
                                    continue;
                                }
                                let p = ctx.core.player(t);
                                if p.hand.len() <= 1 && p.equips.len() <= 1 {
                                    continue;
                                }
                                // 保留首张手牌+首张装备，其余弃置
                                let keep_hand =
                                    ctx.core.player(t).hand.first().copied();
                                let keep_equip =
                                    ctx.core.player(t).equips.first().copied();
                                let extra_hand: Vec<crate::ids::CardId> = ctx
                                    .core
                                    .player(t)
                                    .hand
                                    .iter()
                                    .skip(1)
                                    .copied()
                                    .collect();
                                for c in extra_hand {
                                    ctx.discard_from_hand(t, c);
                                }
                                let extra_equip: Vec<crate::ids::CardId> = ctx
                                    .core
                                    .player(t)
                                    .equips
                                    .iter()
                                    .skip(1)
                                    .copied()
                                    .collect();
                                for c in extra_equip {
                                    ctx.core
                                        .player_mut(t)
                                        .equips
                                        .retain(|x| *x != c);
                                    revoke_equip_skills(
                                        ctx.core,
                                        ctx.catalog,
                                        t,
                                        c,
                                    );
                                    ctx.to_discard(c);
                                }
                                let _ = (keep_hand, keep_equip);
                            }
                        }
                    }
                    // 鬼计（Boss 黑无常，规则 v256）：准备阶段——弃置自
                    // 己判定区一张牌（随机→首张确定性近似登记）
                    if let Some(gjdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_GUIJI)
                    {
                        if ctx.core.has_skill(player, gjdef) {
                            if let Some(c) = ctx.core.player(player).judge.first().copied() {
                                ctx.core.player_mut(player).judge.retain(|x| *x != c);
                                ctx.to_discard(c);
                            }
                        }
                    }
                    // 公正（十殿阎罗五官王，规则 v265）：准备阶段（判定
                    // 阶段开始锚点）若判定区有牌则弃置首张（上游
                    // phaseZhunbeiBegin 判定区 randomGet 弃置→首张确定性
                    // 近似登记；鬼计同款臂）
                    if let Some(gzdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_GONGZHENG)
                    {
                        if ctx.core.has_skill(player, gzdef) {
                            if let Some(c) = ctx.core.player(player).judge.first().copied() {
                                ctx.core.player_mut(player).judge.retain(|x| *x != c);
                                ctx.to_discard(c);
                            }
                        }
                    }
                    // 吸行（Boss 黑无常，规则 v256）：准备阶段——对座位序
                    // 首个已链接的其他存活角色造成 1 点雷伤并回复 1 点体力
                    //（目标自选→首个已链接确定性近似登记）
                    if let Some(xxdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_XIXING)
                    {
                        if ctx.core.has_skill(player, xxdef) {
                            let seats = ctx.core.players.len() as PlayerId;
                            let linked: Vec<PlayerId> = (0..seats)
                                .filter(|&t| {
                                    t != player
                                        && ctx.core.player(t).alive
                                        && ctx.core.player(t).linked
                                })
                                .collect();
                            if let Some(&t) = linked.first() {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(player),
                                    t,
                                    1,
                                    crate::event::DamageNature::Thunder,
                                    None,
                                )));
                                ctx.recover(player, 1, None);
                            }
                        }
                    }
                    // 雷舟（十殿阎罗卞城王，规则 v267）：准备阶段——随机
                    // 一名其他存活角色受 1 点雷伤（randomGet→首个确定性
                    // 近似登记；吸行同位）
                    if let Some(lzdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_LEIZHOU)
                    {
                        if ctx.core.has_skill(player, lzdef) {
                            let seats = ctx.core.players.len() as PlayerId;
                            if let Some(t) =
                                (0..seats).find(|&t| t != player && ctx.core.player(t).alive)
                            {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(player),
                                    t,
                                    1,
                                    crate::event::DamageNature::Thunder,
                                    None,
                                )));
                            }
                        }
                    }
                    // 封冻（Boss 应昭，规则 v228）：回合开始——所有其他
                    // 存活角色获「封印」（本回合无法使用手牌；回合结束解
                    // 除。上游 boss_fengdong phaseBegin addTempSkill fengyin
                    // 同口径）
                    if let Some(fddef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_FENGDONG)
                    {
                        if ctx.core.has_skill(player, fddef) {
                            for t in ctx.core.seat_order_from(player) {
                                if t != player && ctx.core.player(t).alive {
                                    ctx.core.player_mut(t).fengyin_active = true;
                                }
                            }
                        }
                    }
                    // 巡游（Boss 应昭，规则 v231）：其他角色回合开始——
                    // 获得座位序首名有手牌的其他角色的一张手牌（确定性近
                    // 似：hej 全区域/随机以座位序+手牌首张近似登记）；装
                    // 备牌自动使用（直接装上，v180 双刃系 grant 先例）
                    if let Some(xdef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_XUNYOU)
                    {
                        // global phaseBegin：任意 enabled 持有者（≠当前回
                        // 合玩家）在场即触发（炎狱 holders 先例同构）
                        let holders: Vec<PlayerId> = ctx
                            .core
                            .skills
                            .iter()
                            .filter(|((_, dd), i)| *dd == xdef && i.enabled)
                            .map(|((o, _), _)| *o)
                            .filter(|o| {
                                *o != player && ctx.core.player(*o).alive
                            })
                            .collect();
                        if !holders.is_empty() {
                            let holder = holders[0];
                            let mut victim: Option<PlayerId> = None;
                            for t in ctx.core.seat_order_from(holder) {
                                if t != holder
                                    && ctx.core.player(t).alive
                                    && !ctx.core.player(t).hand.is_empty()
                                {
                                    victim = Some(t);
                                    break;
                                }
                            }
                            if let Some(v) = victim {
                                let c = ctx.core.player(v).hand[0];
                                let kind = ctx.catalog.card_kind(c);
                                ctx.core.player_mut(v).hand.retain(|x| *x != c);
                                if let Some(slot) = kind.equip_slot() {
                                    // 装备牌自动使用：同栏替换（旧装备入弃
                                    // 堆并撤销技能——931 区 play 装备先例）
                                    if let Some(old) = ctx
                                        .core
                                        .player(player)
                                        .equips
                                        .iter()
                                        .copied()
                                        .find(|e| {
                                            ctx.catalog.card_kind(*e).equip_slot()
                                                == Some(slot)
                                        })
                                    {
                                        ctx.core.player_mut(holder)
                                            .equips
                                            .retain(|x| *x != old);
                                        revoke_equip_skills(
                                            ctx.core,
                                            ctx.catalog,
                                            holder,
                                            old,
                                        );
                                        ctx.to_discard(old);
                                    }
                                    ctx.core.player_mut(holder).equips.push(c);
                                    grant_equip_skills(ctx, holder, c);
                                } else {
                                    ctx.core.player_mut(holder).hand.push(c);
                                }
                                ctx.log(LogEvent::CardsGiven {
                                    from: v,
                                    to: player,
                                    cards: vec![c],
                                });
                            }
                        }
                    }
                    // 炎狱（Boss 毕方，规则 v229）：global phaseBegin——
                    // 其他角色的回合开始判定（确定性判定铁骑先例），红色 →
                    // 该角色受 1 点来自毕方的火伤；每回合至多 3 次（TurnState
                    // 计数器，回合更替清零；上游 getIndex 3 同口径）
                    if let Some(ydef) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_YANYU)
                    {
                        let holders: Vec<PlayerId> = ctx
                            .core
                            .skills
                            .iter()
                            .filter(|((_, dd), i)| *dd == ydef && i.enabled)
                            .map(|((o, _), _)| *o)
                            .filter(|o| *o != player && ctx.core.player(*o).alive)
                            .collect();
                        if ctx.core.turn.yanyu_count < 3 {
                            for holder in holders {
                                let judged = ctx.draw_judge_card();
                                let red = matches!(
                                    ctx.catalog.cards[judged as usize].suit,
                                    crate::catalog::Suit::Heart
                                        | crate::catalog::Suit::Diamond
                                );
                                ctx.to_discard(judged);
                                if red {
                                    ctx.core.turn.yanyu_count += 1;
                                    ctx.emit(EventData::Damage(
                                        crate::machine::damage_data_with(
                                            Some(holder),
                                            player,
                                            1,
                                            crate::event::DamageNature::Fire,
                                            None,
                                        ),
                                    ));
                                }
                                break; // 每次判定至多一持技者结算（单 Boss 场景）
                            }
                        }
                    }
                    // 中毒（Boss 书精授技，规则 v220）：回合开始判定——
                    // 非红桃受 1 点无来源伤害；非黑桃失去此技（上游
                    // boss_zhongdu phaseZhunbeiBegin judge 同口径；确定性
                    // 判定=铁骑内联先例）
                    if let Some(zd) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_ZHONGDU)
                    {
                        if ctx.core.has_skill(player, zd) {
                            let judged = ctx.draw_judge_card();
                            let suit = ctx.catalog.cards[judged as usize].suit;
                            ctx.to_discard(judged);
                            match suit {
                                crate::catalog::Suit::Heart => {}
                                crate::catalog::Suit::Spade => {
                                    ctx.log(LogEvent::SkillTriggered {
                                        skill: "中毒",
                                        owner: player,
                                    });
                                }
                                _ => {
                                    // 方片/梅花：伤 1 后失去此技
                                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                                        None,
                                        player,
                                        1,
                                    )));
                                    ctx.core.skills.remove(&(player, zd));
                                }
                            }
                        }
                    }
                    ctx.core.turn.yingfeng_chain = false;
                    ctx.core.turn.luanjian_card = None;
                    ctx.core.turn.mujiaren_used = false;
                }
                // 废墟地图效果（mtg 第四批，规则 v103）：锁定全局——回合
                // 开始（准备阶段）随机将弃牌堆一张置于牌堆顶（上游
                // phaseBegin silent 同口径；RNG 真消除口径）
                if ctx.core.current_land == Some(crate::catalog::CardKind::Feixu)
                    && !ctx.core.discard.is_empty()
                {
                    let n = ctx.core.discard.len() as u32;
                    let i = ctx.core.rng.gen_range(n) as usize;
                    let c = ctx.core.discard.remove(i);
                    ctx.core.deck.push(c);
                }
            }
            Phase::Draw => {
                if ctx.core.turn.skip_draw_phase {
                    // 兵粮寸断：跳过摸牌
                    ctx.core.turn.skip_draw_phase = false;
                } else if ctx.core.player(player).skip_next_draw {
                    // 归烬牌（U05 xianjian 明绣，规则 v160）：跳过本次摸
                    // 牌阶段（上游 skip("phaseDraw") 同口径）
                    ctx.core.player_mut(player).skip_next_draw = false;
                } else if ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_JIUSHOU)
                    .map(|def| ctx.core.has_skill(player, def))
                    .unwrap_or(false)
                {
                    // 兽九（Boss 相柳，规则 v222）：跳过摸牌阶段（上游
                    // boss_jiushou phaseDraw trigger.cancel 同口径）
                } else {
                    // 猛态：摸牌阶段已发生（跳摸路径不清位——上游
                    // phaseDrawBegin 未触发同口径）
                    ctx.core.turn.mengtai_draw_pending = false;
                    ctx.draw_cards(player, 2);
                    // 神恩（Boss 太昊/炎帝 光环，规则 v238）：挑战者侧
                    // 摸牌 +1（上游 boss_shenen2 phaseDrawBegin num++ 同
                    // 口径；光环激活=场上存在神恩持有者）
                    {
                        let active = ctx
                            .catalog
                            .skill_by_name(crate::content::standard::BOSS_SHENEN)
                            .map(|def| ctx.core.skills.keys().any(|(_, dd)| *dd == def))
                            .unwrap_or(false);
                        if active && !ctx.core.is_boss_side(player) {
                            ctx.draw_cards(player, 1);
                        }
                    }
                    // 无脏（Boss 混沌，规则 v209）：摸牌基数改为
                    // max(5, floor(hp/2))——在固定摸 2 后补额外量
                    // max(5, hp/2) - 2（上游 phaseDrawBegin trigger.num+=
                    // 同口径）
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_WUZANG)
                    {
                        if ctx.core.has_skill(player, def) {
                            let hp = ctx.core.player(player).hp.max(0) as u32;
                            let extra = hp / 2;
                            let total = extra.max(5);
                            if total > 2 {
                                ctx.draw_cards(player, (total - 2) as usize);
                            }
                        }
                    }
                    // 藏目（Boss 饕餮，规则 v216）：摸牌基数改为存活角色
                    // 数——固定摸 2 后补额外 存活数-2（上游 boss_cangmu
                    // phaseDrawBegin trigger.num += countPlayer()-2 同口径）
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_CANGMU)
                    {
                        if ctx.core.has_skill(player, def) {
                            let alive = ctx.core.alive_count();
                            if alive > 2 {
                                ctx.draw_cards(player, (alive - 2) as usize);
                            }
                        }
                    }
                    // 年瑞（Boss 年兽，规则 v248）：摸牌阶段 +2（上游
                    // boss_nianrui phaseDrawBegin trigger.num += 2 同口径）
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_NIANRUI)
                    {
                        if ctx.core.has_skill(player, def) {
                            ctx.draw_cards(player, 2);
                        }
                    }
                    // 太平（Boss 黑无常，规则 v256）：摸牌阶段 +2（年瑞
                    // 同位）
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_TAIPING)
                    {
                        if ctx.core.has_skill(player, def) {
                            ctx.draw_cards(player, 2);
                        }
                    }
                }
            }
            Phase::Play => {
                // 星囚（Boss 蓐收，规则 v239）：出牌阶段开始交替横置全
                // 部敌方（上游 boss_xingqiu phaseUseBegin storage 取反+
                // content link 同口径；只链不解——已横置保持；对 boss_
                // mingxingzhu 授 boss_jiding 面依赖武将未入库，登记差异）
                if let Some(xqdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_XINGQIU)
                {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == xqdef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    for holder in holders {
                        // 上游 trigger player: phaseUseBegin——仅持有者
                        // 本人出牌阶段触发（他人阶段开始不翻转开关）
                        if holder != player || !ctx.core.player(holder).alive {
                            continue;
                        }
                        let now = !ctx.core.player(holder).xingqiu_toggled;
                        ctx.core.player_mut(holder).xingqiu_toggled = now;
                        if !now {
                            continue;
                        }
                        let seats = ctx.core.players.len() as PlayerId;
                        for t in 0..seats {
                            if t == holder
                                || !ctx.core.player(t).alive
                                || ctx.core.is_boss_side(t) == ctx.core.is_boss_side(holder)
                            {
                                continue;
                            }
                            if !ctx.core.player(t).linked {
                                ctx.core.player_mut(t).linked = true;
                                ctx.log(LogEvent::Linked { player: t, linked: true });
                            }
                        }
                    }
                }
                // 兽九（Boss 相柳，规则 v222）：出牌阶段开始补至 9（上游
                // phaseUseBegin draw(9-count) 同口径）
                if let Some(def) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_JIUSHOU)
                {
                    if ctx.core.has_skill(player, def) {
                        let n = ctx.core.player(player).hand.len();
                        if n < 9 {
                            ctx.draw_cards(player, 9 - n);
                        }
                    }
                }
                ctx.core.turn.sha_used_play_phase = false;
                // 突击计数（U07 ow，规则 v118）：出牌阶段开始清零
                ctx.core.turn.cards_used_play_phase = 0;
                // 猛态：出牌阶段已发生（跳过路径不清位——上游
                // phaseUseBegin 同口径）
                ctx.core.turn.mengtai_use_pending = false;
                // 魔剑（Boss 夜叉，规则 v253）：出牌阶段开始视为对全部
                // 敌方使用【万箭齐发】（虚拟牌机器直发——异侧存活全目标；
                // 上游 canUse 过滤同口径）
                if let Some(mjdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_MOJIAN)
                {
                    if ctx.core.has_skill(player, mjdef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        let targets: Vec<PlayerId> = (0..seats)
                            .filter(|&t| {
                                ctx.core.player(t).alive
                                    && ctx.core.is_boss_side(t)
                                        != ctx.core.is_boss_side(player)
                            })
                            .collect();
                        if !targets.is_empty() {
                            ctx.use_virtual_card(
                                player,
                                CardKind::ArcheryAttack,
                                targets,
                            );
                        }
                    }
                }
                // 御兽（Boss 罗刹，规则 v254）：出牌阶段开始视为对全部敌
                // 方使用【南蛮入侵】（魔剑同款虚拟南蛮）
                if let Some(ysdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_YUSHOU)
                {
                    if ctx.core.has_skill(player, ysdef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        let targets: Vec<PlayerId> = (0..seats)
                            .filter(|&t| {
                                ctx.core.player(t).alive
                                    && ctx.core.is_boss_side(t)
                                        != ctx.core.is_boss_side(player)
                            })
                            .collect();
                        if !targets.is_empty() {
                            ctx.use_virtual_card(
                                player,
                                CardKind::SavageAttack,
                                targets,
                            );
                        }
                    }
                }
            }
            Phase::Discard => {
                // 虚失（Boss 朱厌，规则 v225）：出牌阶段结束自翻面（上游
                // phaseUseEnd turnOver 同口径；锚=弃牌阶段开始[原生无
                // PhaseEnd 帧]；经 TurnOverBefore 走翻面提交）
                if let Some(xdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_XUSHI)
                {
                    if ctx.core.has_skill(player, xdef)
                        && ctx.core.player(player).alive
                    {
                        ctx.emit(EventData::TurnOverBefore {
                            player,
                            cancelled: false,
                        });
                    }
                }
                // 贪欲（Boss 饕餮，规则 v215）：锁定——跳过你的弃牌阶段
                //（上游 boss_tanyu phaseDiscardBefore trigger.cancel 同口径）
                if let Some(def) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_TANYU)
                {
                    if ctx.core.has_skill(player, def) {
                        ctx.core.turn.discard_phase_discarded = 0;
                        return;
                    }
                }
                // 庆祝（Boss 明星，规则 v245）：锁定——跳过你的弃牌阶段
                //（上游 boss_qingzhu phaseDiscardBefore trigger.cancel 同
                // 口径；与急叮解锁无关，恒跳）
                if let Some(def) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_QINGZHU)
                {
                    if ctx.core.has_skill(player, def) {
                        ctx.core.turn.discard_phase_discarded = 0;
                        return;
                    }
                }
                // 水没墓穴计数（mtg 第三批，规则 v102）：弃牌阶段开始清零
                ctx.core.turn.discard_phase_discarded = 0;
            }
            Phase::End => {
                // 爆敛（Boss 白无常/牛头，规则 v255）：结束阶段摸 2
                if let Some(bldef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_BAOLIAN)
                {
                    if ctx.core.has_skill(player, bldef) {
                        ctx.draw_cards(player, 2);
                    }
                }
                // 强征（Boss 白无常，规则 v255）：结束阶段从每名其他有手
                // 牌角色处随机获得一张手牌（随机→首张确定性近似登记）
                if let Some(qzdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_QIANGZHENG)
                {
                    if ctx.core.has_skill(player, qzdef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        for t in 0..seats {
                            if t == player
                                || !ctx.core.player(t).alive
                                || ctx.core.player(t).hand.is_empty()
                            {
                                continue;
                            }
                            let c = ctx.core.player(t).hand[0];
                            ctx.core.player_mut(t).hand.retain(|x| *x != c);
                            ctx.core.player_mut(player).hand.push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from: t,
                                to: player,
                                cards: vec![c],
                            });
                        }
                    }
                }
                // 枭首（Boss 牛头，规则 v255）：结束阶段对座位序首个
                // hp≥self 的其他存活角色造成 3 点伤害（目标自选可放弃→
                // 恒发动确定性近似登记）
                if let Some(xsdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_XIAOSHOU)
                {
                    if ctx.core.has_skill(player, xsdef) {
                        let hp_self = ctx.core.player(player).hp;
                        let seats = ctx.core.players.len() as PlayerId;
                        if let Some(t) = (0..seats).find(|&t| {
                            t != player
                                && ctx.core.player(t).alive
                                && ctx.core.player(t).hp >= hp_self
                        }) {
                            ctx.emit(EventData::Damage(crate::machine::damage_data(
                                Some(player),
                                t,
                                3,
                            )));
                        }
                    }
                }
                // 索命（Boss 黑无常，规则 v256）：结束阶段——横置（链接）
                // 全部未链接的其他存活角色（[1,num] 自选→全部确定性近似
                // 登记；链接机直写+Linked 日志，索链甲同款）
                if let Some(smdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_SUOMING)
                {
                    if ctx.core.has_skill(player, smdef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        for t in 0..seats {
                            if t == player
                                || !ctx.core.player(t).alive
                                || ctx.core.player(t).linked
                            {
                                continue;
                            }
                            ctx.core.player_mut(t).linked = true;
                            ctx.log(LogEvent::Linked { player: t, linked: true });
                        }
                    }
                }
                // 炼狱（Boss 马面，规则 v257）：结束阶段对全部其他存活
                // 角色各造成 1 点火焰伤害（无询问直伤）
                if let Some(lydef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_LIANYU)
                {
                    if ctx.core.has_skill(player, lydef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        for t in 0..seats {
                            if t == player || !ctx.core.player(t).alive {
                                continue;
                            }
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(player),
                                t,
                                1,
                                crate::event::DamageNature::Fire,
                                None,
                            )));
                        }
                    }
                }
                // 血池（十殿阎罗五官王，规则 v265）：结束阶段座位序首个
                // 其他存活角色失去 2 点体力（上游 randomGet→首个确定性近
                // 似登记）
                if let Some(xcdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_XUECHI)
                {
                    if ctx.core.has_skill(player, xcdef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        if let Some(t) =
                            (0..seats).find(|&t| t != player && ctx.core.player(t).alive)
                        {
                            ctx.emit(EventData::LoseHp { player: t, amount: 2 });
                        }
                    }
                }
                // 胜负（十殿阎罗宋帝王，规则 v266）：结束阶段，一名装备
                // 区有坐骑牌的其他角色弃置其中一张（随机目标/随机牌→首
                // 个有牌者/首张确定性近似登记；上游 logSkill 误标黑绳为
                // 上游原样，native 不复刻；equip6 宝物面随宝物栏机制登记）
                if let Some(sfdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_SHENGFU)
                {
                    if ctx.core.has_skill(player, sfdef) {
                        use crate::catalog::CardKind::{HorseMinus, HorsePlus};
                        let seats = ctx.core.players.len() as PlayerId;
                        if let Some(t) = (0..seats).find(|&t| {
                            t != player
                                && ctx.core.player(t).alive
                                && ctx.core.player(t).equips.iter().any(|c| {
                                    matches!(
                                        ctx.catalog.cards[*c as usize].kind,
                                        HorsePlus | HorseMinus
                                    )
                                })
                        }) {
                            let pick = ctx.core.player(t).equips.iter().copied().find(
                                |c| {
                                    matches!(
                                        ctx.catalog.cards[*c as usize].kind,
                                        HorsePlus | HorseMinus
                                    )
                                },
                            );
                            if let Some(c) = pick {
                                ctx.core.player_mut(t).equips.retain(|x| *x != c);
                                ctx.to_discard(c);
                            }
                        }
                    }
                }
                // 雷斧（十殿阎罗卞城王，规则 v267）：结束阶段——随机一
                // 名其他存活角色横置态切换（上游 link() 非恒链同口径；
                // randomGet→首个确定性近似登记）
                if let Some(lfdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_LEIFU)
                {
                    if ctx.core.has_skill(player, lfdef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        if let Some(t) =
                            (0..seats).find(|&t| t != player && ctx.core.player(t).alive)
                        {
                            let cur = ctx.core.player(t).linked;
                            ctx.emit(EventData::Linked {
                                player: t,
                                linked: !cur,
                            });
                        }
                    }
                }
                // 红莲（Boss 周瑜，规则 v263）：结束阶段摸两张牌，并对
                // 全部其他存活角色各造成 1 点火焰伤害（上游 phaseJieshuBegin
                // draw(2)+逐个 damage("fire") 同口径）
                if let Some(hldef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_ZYHONGLIAN)
                {
                    if ctx.core.has_skill(player, hldef) {
                        ctx.draw_cards(player, 2);
                        let seats = ctx.core.players.len() as PlayerId;
                        for t in 0..seats {
                            if t != player && ctx.core.player(t).alive {
                                ctx.emit(EventData::Damage(
                                    crate::machine::damage_data_with(
                                        Some(player),
                                        t,
                                        1,
                                        crate::event::DamageNature::Fire,
                                        None,
                                    ),
                                ));
                            }
                        }
                    }
                }
                // 芳华（Boss 甄姬，规则 v262）：结束阶段若存在翻面角色，
                // 全部翻面角色各失去 1 点体力（上游 get.players 过滤
                // isTurnedOver 逐个 loseHp 同口径）
                if let Some(fhdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_FANGHUA)
                {
                    if ctx.core.has_skill(player, fhdef)
                        && ctx.core.players.iter().any(|p| p.alive && p.face_down)
                    {
                        let seats = ctx.core.players.len() as PlayerId;
                        for t in 0..seats {
                            if ctx.core.player(t).alive
                                && ctx.core.player(t).face_down
                            {
                                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                            }
                        }
                    }
                }
                // 枭雄子技 xiaoxiong_damage（Boss 刘备，规则 v261）：其
                // 他角色的结束阶段，若其本回合未使用过牌（上游 countUsed()
                // ==0→出牌阶段用牌计数==0 登记差异：响应/装备不计）→ 对其
                // 造成 1 点伤害
                if let Some(xxdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_XIAOXIONG)
                {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == xxdef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    if let Some(holder) = holders.first() {
                        if player != *holder
                            && ctx.core.player(*holder).alive
                            && ctx.core.turn.cards_used_play_phase == 0
                        {
                            ctx.emit(EventData::Damage(crate::machine::damage_data(
                                Some(*holder),
                                player,
                                1,
                            )));
                        }
                    }
                }
                // 伤逝子技 shangshix2（Boss 张春华，规则 v260）：结束
                // 阶段若体力大于 1，全体存活角色（含自己）各失去 1 点体
                // 力（上游 group shangshix2 phaseJieshuBegin get.players
                // 全体 loseHp 同口径）
                if let Some(ssdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_SHANGSHIX)
                {
                    if ctx.core.has_skill(player, ssdef)
                        && ctx.core.player(player).hp > 1
                    {
                        let seats = ctx.core.players.len() as PlayerId;
                        for t in 0..seats {
                            if ctx.core.player(t).alive {
                                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                            }
                        }
                    }
                }
                // 鬼谋（Boss 郭嘉，规则 v259）：结束阶段随机一名其他未
                // 混乱存活角色进入混乱状态（boss_mad 置位，其回合收束解
                // 除；随机→座位序首个确定性近似登记）
                if let Some(gmdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_GUIMOU)
                {
                    if ctx.core.has_skill(player, gmdef) {
                        let seats = ctx.core.players.len() as PlayerId;
                        if let Some(t) = (0..seats).find(|&t| {
                            t != player
                                && ctx.core.player(t).alive
                                && !ctx.core.player(t).boss_mad
                        }) {
                            ctx.core.player_mut(t).boss_mad = true;
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "鬼谋",
                                owner: player,
                            });
                        }
                    }
                }
                // 猛态（Boss 年兽，规则 v248）：本回合未摸牌 → 结束阶段
                // 摸 3（上游 boss_mengtai_end phaseJieshuBegin draw(3)
                // 同口径）
                if let Some(mtdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_MENGTAI)
                {
                    if ctx.core.has_skill(player, mtdef)
                        && ctx.core.turn.mengtai_draw_pending
                    {
                        ctx.core.turn.mengtai_draw_pending = false;
                        ctx.draw_cards(player, 3);
                    }
                }
                // 封冻解除（Boss 应昭，规则 v228）：回合结束全清封印
                for p in ctx.core.players.iter_mut() {
                    p.fengyin_active = false;
                }
                // 巨源（Boss 朱厌，规则 v226）：回合结束记录体力（上游
                // phaseAfter storage=hp 同口径；跨回合持久）
                if let Some(jdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_JUYUAN)
                {
                    if ctx.core.has_skill(player, jdef) {
                        let hp = ctx.core.player(player).hp;
                        ctx.core.player_mut(player).juyuan_storage = hp;
                    }
                }
                // 兵先（Boss 朱厌，规则 v224）：其他角色的回合结束时，若
                // 其本回合未使用【杀】（出牌阶段口径=turn.sha_used_play_
                // phase，登记差异）→ 视为对其使用一张【杀】（use_virtual_
                // card，乱常万箭同款）
                if let Some(bdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_BINGXIAN)
                {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == bdef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    for holder in holders {
                        if player == holder
                            || !ctx.core.player(holder).alive
                            || !ctx.core.player(player).alive
                            || ctx.core.turn.sha_used_play_phase
                        {
                            continue;
                        }
                        ctx.use_virtual_card(holder, CardKind::Sha, vec![player]);
                    }
                }
                // 兽九（Boss 相柳，规则 v222）：回合结束补至 9（上游
                // phaseJieshuBegin draw(9-count) 同口径）
                if let Some(jdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_JIUSHOU)
                {
                    if ctx.core.has_skill(player, jdef) {
                        let n = ctx.core.player(player).hand.len();
                        if n < 9 {
                            ctx.draw_cards(player, 9 - n);
                        }
                    }
                }
                // 贪欲（Boss 饕餮，规则 v215）：结束阶段若你的手牌数为
                // 全场最多（不少于任何其他存活角色）→ 失去 1 点体力（上
                // 游 boss_tanyu_hp phaseJieshuBegin isMaxHandcard 同口径）
                if let Some(tydef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_TANYU)
                {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == tydef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    for holder in holders {
                        if ctx.core.player(holder).alive && tanyu_is_max_hand(ctx.core, holder)
                        {
                            ctx.emit(EventData::LoseHp {
                                player: holder,
                                amount: 1,
                            });
                        }
                    }
                }
                // 乱常（Boss 梼杌，规则 v218）：结束阶段视为用【万箭齐
                // 发】（虚拟牌，目标=全部其他存活角色；上游 boss_luanchang_
                // end phaseJieshuBegin useCard wanjian 同口径）
                if let Some(ldef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_LUANCHANG)
                {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == ldef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    for holder in holders {
                        if !ctx.core.player(holder).alive {
                            continue;
                        }
                        let targets: Vec<PlayerId> = ctx
                            .core
                            .seat_order_from(holder)
                            .into_iter()
                            .filter(|p| *p != holder)
                            .collect();
                        ctx.use_virtual_card(
                            holder,
                            CardKind::ArcheryAttack,
                            targets,
                        );
                    }
                }
                // 延妒（Boss 穷奇，规则 v213）：global phaseJieshuBegin——
                // 其他角色结束阶段开始时，若其本回合未造成伤害且有手牌
                // → 穷奇获得其一张手牌（确定性首张；上游 gainPlayerCard
                // 由穷奇选牌，登记差异）
                if let Some(dudef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_YANDU)
                {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == dudef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    for holder in holders {
                        if player == holder || !ctx.core.player(holder).alive {
                            continue;
                        }
                        let pl = ctx.core.player(player);
                        if pl.alive && pl.turn_damage == 0 && !pl.hand.is_empty() {
                            let c = pl.hand[0];
                            ctx.core.player_mut(player).hand.retain(|x| *x != c);
                            ctx.core.player_mut(holder).hand.push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from: player,
                                to: holder,
                                cards: vec![c],
                            });
                        }
                    }
                }
            }
        }
        return;
    }
    // PhaseEnd
    // 筑城开关（U07 批三，规则 v113）：刘馥自己每阶段结束确认翻转（上
    // 游 player:"phaseEnd" optional 同口径；原生无 begin:false 帧，直推
    // OptionalGate）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::content::host::WUDIAN_ZHUCHENG_KEY)
    {
        if ctx.core.has_skill(player, def) {
            let event = EventData::Phase {
                player,
                phase,
                begin: false,
            };
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger { frame: crate::resolution::NO_FRAME },
                machine: Box::new(crate::resolution::OptionalGate::new(
                    player, def, event, false, None,
                )),
            });
        }
    }
    // 水没墓穴地图效果（mtg 第三批，规则 v102）：锁定——本弃牌阶段弃
    // 置过牌则阶段结束摸一（上游 phaseDiscardEnd forced 同口径）
    if phase == Phase::Discard {
        if ctx.core.current_land == Some(crate::catalog::CardKind::Shuimomuxue)
            && ctx.core.turn.discard_phase_discarded > 0
        {
            ctx.draw_cards(player, 1);
        }
        ctx.core.turn.discard_phase_discarded = 0;
    }
    // 光塔（U07 ow，规则 v119）：♠子技持有者于下阶段结束受 1 无源雷伤
    //（一次性移除；原生无 PhaseEnd 帧——直推同水没墓穴口径）
    {
        if let Some(def) = ctx
            .catalog
            .skill_by_name(crate::content::host::WUDIAN_GUANGSHU_SPADE_KEY)
        {
            if ctx.core.has_skill(player, def) {
                ctx.core.skills.remove(&(player, def));
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    None,
                    player,
                    1,
                    crate::event::DamageNature::Thunder,
                    None,
                )));
            }
        }
    }
    // 乐不思蜀生效：摸牌阶段结束时若跳过出牌旗标在位，
    // 直接进入弃牌阶段（跳过 Play 的 End/Begin 事件对——挂账见清单）。
    if phase == Phase::Draw && ctx.core.turn.skip_play_phase {
        ctx.core.turn.skip_play_phase = false;
        ctx.core.turn.phase = Phase::Discard;
        ctx.emit(EventData::Phase {
            player,
            phase: Phase::Discard,
            begin: true,
        });
        return;
    }
    match phase.next() {
        Some(next) => {
            ctx.core.turn.phase = next;
            ctx.emit(EventData::Phase {
                player,
                phase: next,
                begin: true,
            });
        }
        None => {
            // 混乱解除（Boss 郭嘉鬼谋，规则 v259）：混乱者回合收束解除
            //（上游 addTempSkill "mad" {player:"phaseAfter"} 同口径）；
            // 奇佐副本载体槽跨回合兜底清
            ctx.core.player_mut(player).boss_mad = false;
            ctx.core.turn.qizuo_copy = None;
            ctx.emit(EventData::TurnEnded { player });
        }
    }
}

/// E03 翻面提交：取消位为真则跳过翻转（漩涡面具等锁定技）；
/// 否则翻转武将牌朝向并发出完成事件。
/// 雷击（Boss 青龙，规则 v234）：打出/使用闪后触发——座位序下一名其
/// 他存活角色确定性判定（铁骑内联口径，无鬼道/天妒改判窗；目标选择以
/// 上游 chooseTarget→座位序首名确定性近似登记）：黑桃 → 其受 2 点雷伤；
/// 梅花 → 持有者回复 1、其受 1 点雷伤；其他无事。
fn releiji_on_shan(ctx: &mut Ctx<'_>, holder: PlayerId) {
    let releiji = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_RELEIJI)
        .map(|d| ctx.core.has_skill(holder, d))
        .unwrap_or(false);
    if !releiji {
        return;
    }
    let tgt = ctx.core.next_alive(holder);
    if tgt == holder || !ctx.core.player(tgt).alive {
        return;
    }
    let judged = ctx.draw_judge_card();
    let suit = ctx.catalog.cards[judged as usize].suit;
    ctx.to_discard(judged);
    match suit {
        crate::catalog::Suit::Spade => {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                None,
                tgt,
                2,
                crate::event::DamageNature::Thunder,
                None,
            )));
        }
        crate::catalog::Suit::Club => {
            ctx.recover(holder, 1, None);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                None,
                tgt,
                1,
                crate::event::DamageNature::Thunder,
                None,
            )));
        }
        _ => {}
    }
}

fn commit_turn_over_before(ctx: &mut Ctx<'_>, player: PlayerId, cancelled: bool) {
    if cancelled {
        return;
    }
    // 凶兽（Boss 模式共有技，规则 v208）：你不能被翻面（上游
    // boss_xiongshou_turn turnOverBefore priority 20 trigger.cancel 同
    // 口径——入口直读跳过，死亡取消型基建同款）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_XIONGSHOU)
    {
        if ctx
            .core
            .skills
            .get(&(player, def))
            .map(|i| i.enabled)
            .unwrap_or(false)
        {
            return;
        }
    }
    // 神裔（Boss 青龙共有技，规则 v233）：武将牌不能翻至背面（上游
    // boss_shenyi turnOverBefore filter !isTurnedOver——仅正→背取消，
    // 背→正照常翻回）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_SHENYI)
    {
        if ctx
            .core
            .skills
            .get(&(player, def))
            .map(|i| i.enabled)
            .unwrap_or(false)
            && !ctx.core.player(player).face_down
        {
            return;
        }
    }
    let face_down = !ctx.core.player(player).face_down;
    ctx.core.player_mut(player).face_down = face_down;
    // 急冻冰封解除（U05 ow 美，规则 v127）：翻回正面即冰封结束——上游
    // jidong2 至 turnOverAfter 到期同口径（含翻面跳回合的翻回；急冻
    // 自身翻至背面为内联翻转不经此处）
    if !face_down {
        ctx.core.player_mut(player).jidong_active = false;
        // 虚失（Boss 朱厌，规则 v225）：武将牌翻回正面时对所有其他角色
        // 各造成 1 点伤害（上游 [1,2] 随机——恒 1 登记差异；激活态查询）
        if let Some(xdef) = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_XUSHI)
        {
            if ctx.core.has_skill(player, xdef) {
                for t in ctx.core.seat_order_from(player) {
                    if t != player && ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(player),
                            t,
                            1,
                        )));
                    }
                }
            }
        }
    }
    ctx.emit(EventData::TurnOvered { player, face_down });
}

/// 胆术/精觉（Boss 夜叉/精觉年兽，规则 v252）：玩家失去手牌后（上游
/// loseAfter+loseAsync 族的 CardsDiscarded 面；gain/addJudge/equip 等
/// 其余卡牌移动面登记差异）——非当前回合且已受伤 → 判定红色回 1（铁骑
/// 内联确定性口径，无鬼才/天妒改判窗）。
fn commit_cards_discarded(ctx: &mut Ctx<'_>, player: PlayerId) {
    let names = [
        crate::content::standard::BOSS_DANSHU,
        crate::content::standard::BOSS_JINGJUE,
    ];
    for name in names {
        let Some(def) = ctx.catalog.skill_by_name(name) else {
            continue;
        };
        if !ctx.core.has_skill(player, def) {
            continue;
        }
        // 非当前回合 + 已受伤
        if ctx.core.turn.current == player || !ctx.core.player(player).alive {
            continue;
        }
        let p = ctx.core.player(player);
        if p.hp >= p.max_hp as i32 {
            continue;
        }
        let judged = ctx.draw_judge_card();
        let red = matches!(
            ctx.catalog.cards[judged as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
        ctx.to_discard(judged);
        if red {
            ctx.recover(player, 1, None);
        }
        break; // 胆术/精觉同内容不叠加
    }
    // 魔炎（Boss 罗刹，规则 v254）：非当前回合失去牌后判定红色 → 对座
    // 位序首个其他存活角色造成 2 点火焰伤害（目标自选→确定性近似登记；
    // loseEnd 全牌损失面→CardsDiscarded 面登记差异）
    if let Some(mydef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_MOYANY)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == mydef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| {
                ctx.core.player(*o).alive && ctx.core.turn.current != *o
            })
            .collect();
        if let Some(holder) = holders.first() {
            let judged = ctx.draw_judge_card();
            let red = matches!(
                ctx.catalog.cards[judged as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            ctx.to_discard(judged);
            if red {
                let seats = ctx.core.players.len() as PlayerId;
                if let Some(t) = (0..seats).find(|&t| {
                    t != *holder
                        && ctx.core.player(t).alive
                        && ctx.core.is_boss_side(t)
                            != ctx.core.is_boss_side(*holder)
                }) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(*holder),
                        t,
                        2,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                }
            }
        }
    }
    // 绝策（Boss 马面，规则 v258，一将包李儒旧绝策）：自己回合内其他角
    // 色失去最后一手牌（弃后空手）→ 对其造成 1 点伤害（chooseBool AI→
    // 恒发动确定性近似登记；lose/equip/gain 全事件族→CardsDiscarded 面
    // 登记差异）
    if let Some(jcdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_JUECE)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == jcdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| {
                ctx.core.player(*o).alive
                    && ctx.core.turn.current == *o
                    && *o != player
            })
            .collect();
        for holder in holders {
            if ctx.core.player(player).hand.is_empty() {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(holder),
                    player,
                    1,
                )));
            }
        }
    }
    // 仙音（Boss 周瑜，规则 v263）：非当前回合失去牌后判定红 → 座位序
    // 首个其他存活角色失去 1 点体力（上游 loseEnd frequent judge 红 →
    // chooseTarget loseHp 同口径；chooseTarget AI 态度面→首个确定性近似
    // 登记）
    if let Some(xydef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_XIANYIN)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == xydef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| {
                ctx.core.player(*o).alive && ctx.core.turn.current != *o
            })
            .collect();
        if let Some(holder) = holders.first() {
            let judged = ctx.draw_judge_card();
            let red = matches!(
                ctx.catalog.cards[judged as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            ctx.to_discard(judged);
            if red {
                let seats = ctx.core.players.len() as PlayerId;
                if let Some(t) = (0..seats)
                    .find(|&t| t != *holder && ctx.core.player(t).alive)
                {
                    ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                }
            }
        }
    }
    // 伤逝（规则 v260）：失去牌后补牌
    shangshix_draw(ctx, player);
}

/// 造异异变 def 查询（运行时授予技）。
fn zhp_def(ctx: &Ctx<'_>) -> crate::ids::SkillDefId {
    ctx.catalog
        .skill_by_name(crate::content::standard::BOSS_ZAOYI_HP)
        .expect("造异异变入目录")
}

/// 回合收束：到期技能移除，然后交给下一位存活玩家并开启判定阶段。
fn commit_turn_ended(ctx: &mut Ctx<'_>, player: PlayerId) {
    ctx.log(LogEvent::TurnEnded { player });
    ctx.expire_temporary_skills(player);
    // 清残留跳阶段旗标（正常流程已被消费）
    ctx.core.turn.skip_draw_phase = false;
    ctx.core.turn.skip_play_phase = false;
    // Boss 模式（P2/N08 模式入口批三-a，规则 v82）：死亡座位不跳过——
    // 轮到其回合时执行重整 tick（上游 bossPhaseLoop）：计数 +1、hp<上限
    // 则 +1（否则手牌 <4 摸 1），计数达阈值复活；连续死亡座位逐个 tick
    // 后落在下一存活座位。
    if let Some((_, threshold)) = ctx.core.boss_mode() {
        let seats = ctx.core.players.len() as u8;
        let mut next = (player + 1) % seats;
        loop {
            if ctx.core.player(next).alive {
                break;
            }
            if threshold == 0 {
                // 上游 game.bossinfo.chongzheng 未设：无复活模式，跳过 tick
                next = (next + 1) % seats;
                if next == (player + 1) % seats {
                    break;
                }
                continue;
            }
            // 重整 tick
            let cnt = ctx.core.player(next).boss_chongzheng + 1;
            ctx.core.player_mut(next).boss_chongzheng = cnt;
            if (ctx.core.player(next).hp as u32) < ctx.core.player(next).max_hp {
                ctx.core.player_mut(next).hp += 1;
            } else if ctx.core.player(next).hand.len() < 4 {
                ctx.draw_cards(next, 1);
            }
            ctx.log(LogEvent::BossChongzhengTick { player: next, count: cnt });
            if cnt >= threshold {
                let p = ctx.core.player_mut(next);
                p.alive = true;
                p.dying = false;
                p.boss_chongzheng = 0;
                ctx.log(LogEvent::BossRevived { player: next });
            }
            next = (next + 1) % seats;
            if next == (player + 1) % seats {
                break; // 全场死亡（终局判负已在死亡结算处理）
            }
        }
        if next <= player {
            ctx.core.turn.round += 1;
        }
        ctx.core.turn.current = next;
        ctx.core.turn.phase = Phase::Judge;
        ctx.core.turn.sha_used_play_phase = false;
        ctx.core.turn.land_used = false;
        ctx.core.turn.land_sha_used = false;
        ctx.core.turn.turn_opened = false;
        // 回合级存量清零（v128 修复 v126 遗留：TurnState 换回合为原地
        // 改字段而非重建——乐动三角/链钩必须显式复位）
        ctx.core.turn.yuedong_num = 0;
        ctx.core.turn.yuedong_eff = 0;
        ctx.core.turn.yuedong_recover = false;
        ctx.core.turn.liangou_hook = None;
        ctx.core.turn.liangou_bonus_used = false;
        ctx.core.turn.feitiao_target = None;
        ctx.core.turn.fanling_used = false;
        ctx.core.turn.xjyufeng_used = 0;
        ctx.core.turn.diewu_draw_used = false;
        ctx.core.turn.lueying_used = false;
    // 冥顽锁定集（Boss 梼杌，规则 v219）：回合更替清零（上游 phaseAfter
    // clear 同窗）
    ctx.core.turn.minwan_targets.clear();
    ctx.core.turn.yanyu_count = 0;
        ctx.core.turn.txianqu_used = false;
        ctx.core.turn.jtjubao_used = false;
        ctx.emit(EventData::Phase {
            player: next,
            phase: Phase::Judge,
            begin: true,
        });
        return;
    }
    let next = ctx.core.next_alive(player);
    if next <= player {
        ctx.core.turn.round += 1;
    }
    ctx.core.turn.current = next;
    ctx.core.turn.phase = Phase::Judge;
    ctx.core.turn.sha_used_play_phase = false;
    ctx.core.turn.land_used = false;
    ctx.core.turn.land_sha_used = false;
    ctx.core.turn.turn_opened = false;
    // 回合级存量清零（v128 修复 v126 遗留：TurnState 换回合为原地
    // 改字段而非重建——乐动三角/链钩必须显式复位）
    ctx.core.turn.yuedong_num = 0;
    ctx.core.turn.yuedong_eff = 0;
    ctx.core.turn.yuedong_recover = false;
    ctx.core.turn.liangou_hook = None;
    ctx.core.turn.liangou_bonus_used = false;
    ctx.core.turn.feitiao_target = None;
    ctx.core.turn.fanling_used = false;
    ctx.core.turn.xjyufeng_used = 0;
    ctx.core.turn.diewu_draw_used = false;
    ctx.core.turn.lueying_used = false;
    // 冥顽锁定集（Boss 梼杌，规则 v219）：回合更替清零（上游 phaseAfter
    // clear 同窗）
    ctx.core.turn.minwan_targets.clear();
    ctx.core.turn.yanyu_count = 0;
    ctx.core.turn.txianqu_used = false;
    ctx.core.turn.jtjubao_used = false;
    ctx.emit(EventData::Phase {
        player: next,
        phase: Phase::Judge,
        begin: true,
    });
}

/// 帧固有流程机器（`Intrinsic` 阶段启动）。
pub fn intrinsic_machine(ctx: &mut Ctx<'_>, frame_id: EventId) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let data = ctx.res.frame(frame_id).expect("frame").data.clone();
    match data {
        EventData::LoseHp { .. } => None,
        EventData::DyingSaved { .. } => None,
        EventData::Dying { player, source } => Some(Box::new(DyingSaveMachine {
            dying: player,
            source,
            responders: ctx.core.seat_order_from(ctx.core.turn.current),
            asked: false,
            round_used: false,
        })),
        EventData::CardUsed(u) => {
            // 筑城守卫（U07 批三，规则 v113）等 AfterTriggers 机器写回的
            // 取消：跳过结算（杀已由守卫归还手牌）
            if u.cancelled {
                return None;
            }
            let effective_kind = u
                .use_as
                .unwrap_or_else(|| ctx.catalog.card_kind(u.card));
            match effective_kind {
            CardKind::Dismantle => {
                let effect = DismantleMachine {
                    card: u.card,
                    source: u.player,
                    target: *u.targets.first().expect("dismantle has one target"),
                    asked: false,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            // 无中生有（P2/N08 缺牌种批①）：使用后摸 2（无懈窗口包装）。
            CardKind::Wuzhong => Some(Box::new(wrap_nullify_window(
                ctx,
                u.card,
                u.player,
                Box::new(WuzhongMachine { owner: u.player, card: Some(u.card) }),
            ))),
            // 桃园结义（P2/N08 缺牌种批①）：所有其他存活角色回复 1 点
            // 体力（无懈窗口包装）。
            CardKind::Taoyuan => Some(Box::new(wrap_nullify_window(
                ctx,
                u.card,
                u.player,
                Box::new(TaoyuanMachine { owner: u.player, card: Some(u.card) }),
            ))),
            // 五谷丰登（批①-2）：亮存活角色数的牌，从使用者起按座位序
            // 逐人择一（无懈窗口包装）。
            CardKind::Wugu => Some(Box::new(wrap_nullify_window(
                ctx,
                u.card,
                u.player,
                Box::new(WuguMachine {
                    owner: u.player,
                    cards: Vec::new(),
                    pickers: Vec::new(),
                    stage: WuguStage::Init,
                }),
            ))),
            // 借刀杀人（批①-2）：武器持有者出杀（登记差异：直接伤害
            // 结算，无出闪窗口）或交出武器（无懈窗口包装）。
            CardKind::Jiedao => {
                let holder = *u.targets.first().expect("jiedao two targets");
                let victim = *u.targets.get(1).expect("jiedao two targets");
                Some(Box::new(wrap_nullify_window(
                    ctx,
                    u.card,
                    u.player,
                    Box::new(JiedaoMachine {
                        owner: u.player,
                        holder,
                        victim,
                        card: Some(u.card),
                        stage: JiedaoStage::Ask,
                    }),
                )))
            }
            CardKind::Snatch => {
                let effect = SnatchMachine {
                    card: u.card,
                    source: u.player,
                    target: *u.targets.first().expect("snatch has one target"),
                    asked: false,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
                // 方天画戟（批③-b 第九把）：多目标队列（首目标即期结算，
                // 其余入队列逐目标推进）。
                let mut it = u.targets.iter().copied();
                let first = it.next().expect("sha has one target");
                let extra: Vec<PlayerId> = it.collect();
                Some(Box::new(ShaResolveMachine {
                card: u.card,
                source: u.player,
                target: first,
                nature: u.nature,
                asked: false,
                armor_judged: false,
                dodges_used: 0,
                tieji_red: None,
                liuli_done: false,
                liuli_stage: 0,
                liuli_card: None,
                asking_qilin: false,
                qilin_horse: None,
                asking_hanbing: false,
                asking_fengxueren: false,
                asking_guanshi: false,
                guanshi_cards: Vec::new(),
                asking_qinglong: false,
                asking_cixiong: false,
                cixiong_done: false,
                hanbing_pending: 0,
                asking_xiandan: false,
                xiandan_done: false,
                xiandan_bonus: false,
                asking_lueying: false,
                lueying_victim: None,
                lueying_declined: false,
                extra_targets: extra,
                hujia_queue: None,
                hujia_holder: None,
                // 酒效果在杀派发（=使用）时捕获并消耗——「下一张杀」
                jiu_bonus: std::mem::take(&mut ctx.core.turn.jiu_active),
            }))
            }
            CardKind::Shuigong => {
                // 水攻（G1 运筹帷幄，锦囊）：无懈窗口 + 多目标强制弃装备
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::WD_SHUIGONG_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::ShuigongCard,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: u.targets.clone(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Shezhanqunru => {
                // 舌战群儒（G1 运筹帷幄，锦囊）：与所有有手牌的其他角色拼点
                //（登记差异：确定性首轮拼点，简化多轮 Bool 链）
                let targets: Vec<u8> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != u.player && !ctx.core.player(*p).hand.is_empty())
                    .collect();
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::WD_SHEZHAN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::ShezhanqunruCompare,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets,
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Shujinsan => {
                // 舒筋散（G1 古剑，锦囊）：目标弃任意张手牌（可取消）摸等量
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::GJ_SHUJINSAN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::GujianShujinsanCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("shujinsan has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::SelectCard,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Tanhuadong
            | CardKind::Yougeng
            | CardKind::Qingtuan
            | CardKind::Yuanbaorou
            | CardKind::Gudonggeng
            | CardKind::Mapodoufu
            | CardKind::Chunbing
            | CardKind::Liyutang
            | CardKind::Molicha
            | CardKind::Xiajiao
            | CardKind::Mizhilianou
            | CardKind::Luyugeng
            | CardKind::Ziyangdan
            | CardKind::Yunvyuanshen => {
                // 古剑食物（G1）：授予目标监听技实例并弃置食物牌
                //（上游食物无 wuxieable，无懈窗口不包）
                let key = match effective_kind {
                    CardKind::Tanhuadong => crate::content::host::GJ_TANHUADONG_CARD_KEY,
                    CardKind::Qingtuan => crate::content::host::GJ_QINGTUAN_CARD_KEY,
                    CardKind::Yuanbaorou => crate::content::host::GJ_YUANBAOROU_CARD_KEY,
                    CardKind::Gudonggeng => crate::content::host::GJ_GUDONGGENG_CARD_KEY,
                    CardKind::Mapodoufu => crate::content::host::GJ_MAPODOUFU_CARD_KEY,
                    CardKind::Chunbing => crate::content::host::GJ_CHUNBING_CARD_KEY,
                    CardKind::Liyutang => crate::content::host::GJ_LIYUTANG_CARD_KEY,
                    CardKind::Molicha => crate::content::host::GJ_MOLICHA_CARD_KEY,
                    CardKind::Xiajiao => crate::content::host::GJ_XIAJIAO_CARD_KEY,
                    CardKind::Mizhilianou => crate::content::host::GJ_MIZHLIANOU_CARD_KEY,
                    CardKind::Luyugeng => crate::content::host::GJ_LUYUGENG_CARD_KEY,
                    CardKind::Bingpotong => crate::content::host::GJ_BINGPOTONG_CARD_KEY,
                    CardKind::Qiankunbiao => crate::content::host::GJ_QIANKUNBIAO_CARD_KEY,
                    CardKind::Longxugou => crate::content::host::GJ_LONGXUGOU_CARD_KEY,
                    CardKind::Shenhuofeiya => crate::content::host::GJ_SHENHUOFEIYA_CARD_KEY,
                    CardKind::Ziyangdan => crate::content::host::GJ_ZIYANGDAN_CARD_KEY,
                    CardKind::Yunvyuanshen => crate::content::host::GJ_YUNVYUANSHEN_CARD_KEY,
                    _ => crate::content::host::GJ_YOUGENG_CARD_KEY,
                };
                let behavior = crate::content::host::HostBehavior::GujianFoodGrant;
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("food has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Dinvxuanshuang => {
                // 帝女玄霜（G1 古剑）：主动使用——回复 1 + 弃任意摸等量
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::GJ_DINVXUANSHUANG_CARD_KEY
                        .to_string(),
                    behavior: crate::content::host::HostBehavior::JiguanDinvCard,
                    owner: u.player,
                    target: Some(u.player),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Qinpendayu | CardKind::Birinongwu | CardKind::Ciguhanshuang => {
                // gwent 天气（G1）：目标 + 距离 1 邻座（座位序，已持有
                // 者授予时跳过）；无懈窗口包装
                let t = *u.targets.first().expect("weather has one target");
                let mut q = vec![t];
                for p in ctx.core.alive_players() {
                    if p != t && ctx.core.computed_distance(ctx.catalog, t, p) == 1 {
                        q.push(p);
                    }
                }
                let (key, behavior) = match effective_kind {
                    CardKind::Qinpendayu => (
                        crate::content::host::GW_QINPEN_CARD_KEY,
                        crate::content::host::HostBehavior::GwentWeatherGrant,
                    ),
                    CardKind::Birinongwu => (
                        crate::content::host::GW_BIRIN_CARD_KEY,
                        crate::content::host::HostBehavior::GwentWeatherGrant,
                    ),
                    _ => (
                        crate::content::host::GW_CIGU_CARD_KEY,
                        crate::content::host::HostBehavior::GwentWeatherGrant,
                    ),
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: q,
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Zumoshoukao => {
                // 祖墨手铠（G1 gwent）：无懈窗口 + 封印授予机
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::GW_ZUMO_CARD_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::GwentZumoCard,
                    owner: u.player,
                    target: u.targets.first().copied(),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Kunenfayin | CardKind::Shanbengshu | CardKind::Poxiao => {
                // gwent 收官（G1）：无懈窗口包装 + 各自机器
                let (key, behavior) = match effective_kind {
                    CardKind::Kunenfayin => (
                        crate::content::host::GW_KUNEN_CARD_KEY,
                        crate::content::host::HostBehavior::GwentKunenCard,
                    ),
                    CardKind::Shanbengshu => (
                        crate::content::host::GW_SHANBENG_CARD_KEY,
                        crate::content::host::HostBehavior::GwentShanbengCard,
                    ),
                    _ => (
                        crate::content::host::GW_POXIAO_CARD_KEY,
                        crate::content::host::HostBehavior::GwentPoxiaoCard,
                    ),
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target: Some(u.player),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Yanziyaoshui
            | CardKind::Aozuzhilei
            | CardKind::Wenyi
            | CardKind::Zhihuanjun
            | CardKind::Baoxueyaoshui => {
                // gwent 法术（G1）：无懈窗口包装 + 各自机器
                let (key, behavior) = match effective_kind {
                    CardKind::Yanziyaoshui => (
                        crate::content::host::GW_YANZHI_CARD_KEY,
                        crate::content::host::HostBehavior::GwentYanzhiCard,
                    ),
                    CardKind::Aozuzhilei => (
                        crate::content::host::GW_AOZU_CARD_KEY,
                        crate::content::host::HostBehavior::GwentAozuCard,
                    ),
                    CardKind::Wenyi => (
                        crate::content::host::GW_WENYI_CARD_KEY,
                        crate::content::host::HostBehavior::GwentWenyiCard,
                    ),
                    CardKind::Zhihuanjun => (
                        crate::content::host::GW_ZIHUAN_CARD_KEY,
                        crate::content::host::HostBehavior::GwentZhihuanCard,
                    ),
                    _ => (
                        crate::content::host::GW_BAOXUE_CARD_KEY,
                        crate::content::host::HostBehavior::GwentBaoxueCard,
                    ),
                };
                // 暴雪药水先开目标选牌询问（逐技对照批五 v79），其余直接执行
                let stage = if behavior == crate::content::host::HostBehavior::GwentBaoxueCard {
                    crate::content::host::HostStage::SelectCard
                } else {
                    crate::content::host::HostStage::Act
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target: u.targets.first().copied(),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Shatang | CardKind::Heilonglinpian | CardKind::Liutouge => {
                // 古剑最后机关/锦囊（G1）
                let (key, behavior, target) = match effective_kind {
                    CardKind::Shatang => (
                        crate::content::host::GJ_SHATANG_CARD_KEY,
                        crate::content::host::HostBehavior::JiguanShatangCard,
                        u.targets.first().copied(),
                    ),
                    CardKind::Heilonglinpian => (
                        crate::content::host::GJ_HEILONGLINPIAN_CARD_KEY,
                        crate::content::host::HostBehavior::JiguanHeilongCard,
                        Some(u.player),
                    ),
                    _ => (
                        crate::content::host::GJ_LIUTOUGE_CARD_KEY,
                        crate::content::host::HostBehavior::JiguanLiutougeCard,
                        u.targets.first().copied(),
                    ),
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Dunpaigedang
            | CardKind::Zhiliaobo
            | CardKind::Yuansuhuimie
            | CardKind::Xingjiegoutong
            | CardKind::Chuansongmen
            | CardKind::Tanshezhiren
            | CardKind::Shijieshu => {
                // 炉石余量（G1）：无懈窗口包装 + 各自机器
                let (key, behavior, target, targets) = match effective_kind {
                    CardKind::Dunpaigedang => (
                        crate::content::host::HH_DUNPAI_CARD_KEY,
                        crate::content::host::HostBehavior::HearthDunpaiCard,
                        Some(u.player),
                        Vec::new(),
                    ),
                    CardKind::Zhiliaobo => (
                        crate::content::host::HH_ZHILIAOBO_CARD_KEY,
                        crate::content::host::HostBehavior::HearthZhiliaoboCard,
                        u.targets.first().copied(),
                        Vec::new(),
                    ),
                    CardKind::Yuansuhuimie => (
                        crate::content::host::HH_YUANSU_CARD_KEY,
                        crate::content::host::HostBehavior::HearthYuansuCard,
                        None,
                        ctx.core
                            .alive_players()
                            .filter(|p| *p != u.player)
                            .collect(),
                    ),
                    CardKind::Xingjiegoutong => (
                        crate::content::host::HH_XINGJIE_CARD_KEY,
                        crate::content::host::HostBehavior::HearthXingjieCard,
                        Some(u.player),
                        Vec::new(),
                    ),
                    CardKind::Chuansongmen => (
                        crate::content::host::HH_CHUANSONG_CARD_KEY,
                        crate::content::host::HostBehavior::HearthChuansongCard,
                        Some(u.player),
                        Vec::new(),
                    ),
                    CardKind::Shijieshu => (
                        crate::content::host::HH_SHIJIE_CARD_KEY,
                        crate::content::host::HostBehavior::HearthShijieshuCard,
                        Some(u.player),
                        Vec::new(),
                    ),
                    _ => (
                        crate::content::host::HH_TANSHE_CARD_KEY,
                        crate::content::host::HostBehavior::HearthTansheCard,
                        Some(u.player),
                        Vec::new(),
                    ),
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets,
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Shihuifen | CardKind::Liufengsan => {
                // 古剑锦囊（G1）：石会粉（响应拒绝）/六风散（获得闪）
                let (key, behavior) = match effective_kind {
                    CardKind::Shihuifen => (
                        crate::content::host::GJ_SHIHUIFEN_CARD_KEY,
                        crate::content::host::HostBehavior::GujianShihuifenCard,
                    ),
                    _ => (
                        crate::content::host::GJ_LIUFENGSAN_CARD_KEY,
                        crate::content::host::HostBehavior::GujianLiufengsanCard,
                    ),
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target: u.targets.first().copied(),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Bingpotong
            | CardKind::Qiankunbiao
            | CardKind::Longxugou
            | CardKind::Shenhuofeiya => {
                // 古剑机关（G1）：无懈窗口包装 + 各自机器
                let (key, behavior, targets) = match effective_kind {
                    CardKind::Bingpotong => (
                        crate::content::host::GJ_BINGPOTONG_CARD_KEY,
                        crate::content::host::HostBehavior::JiguanBingpoCard,
                        u.targets.clone(),
                    ),
                    CardKind::Qiankunbiao => (
                        crate::content::host::GJ_QIANKUNBIAO_CARD_KEY,
                        crate::content::host::HostBehavior::JiguanQiankunCard,
                        Vec::new(),
                    ),
                    CardKind::Longxugou => (
                        crate::content::host::GJ_LONGXUGOU_CARD_KEY,
                        crate::content::host::HostBehavior::JiguanLongxuCard,
                        Vec::new(),
                    ),
                    _ => {
                        // 神火飞鸦：目标 + 距离 1 邻座（座位序）响应队列
                        let t = *u.targets.first().expect("shenhuo has one target");
                        let mut q = vec![t];
                        for p in ctx.core.alive_players() {
                            if p != t
                                && p != u.player
                                && ctx.core.computed_distance(ctx.catalog, t, p) == 1
                            {
                                q.push(p);
                            }
                        }
                        (
                            crate::content::host::GJ_SHENHUOFEIYA_CARD_KEY,
                            crate::content::host::HostBehavior::JiguanShenhuoCard,
                            q,
                        )
                    }
                };
                // 冰魄通先开使用者展示询问（逐技对照批五 v79），其余直接执行
                let stage = if behavior == crate::content::host::HostBehavior::JiguanBingpoCard {
                    crate::content::host::HostStage::ComparePlayerPick
                } else if behavior == crate::content::host::HostBehavior::JiguanLongxuCard {
                    crate::content::host::HostStage::SelectCard
                } else {
                    crate::content::host::HostStage::Act
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: key.to_string(),
                    behavior,
                    owner: u.player,
                    target: u.targets.first().copied(),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets,
                    stage,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Liuxinghuoyu => {
                // 流星火雨（U05 swd 首批，规则 v86）：无懈窗口包装 + 目标
                // 自选弃两张 he 或受 1 点火伤
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::SWD_LIUXING_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::SwdLiuxinghuoyuCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("liuxinghuoyu has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Hslingjian => {
                // 灵剑（swd 机关术批，规则 v198）：七卡按名分流；无懈窗
                // 包装共享（上游 wuxieable 同口径）。
                let name = &ctx.catalog.cards[u.card as usize].name;
                let (behavior, def_key, target) = if name.contains("xuanfengzhiren") {
                    (
                        crate::content::host::HostBehavior::SwdXuanfengzhirenCard,
                        crate::content::host::SWD_XUANFENG_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("zhongxinghujia") {
                    (
                        crate::content::host::HostBehavior::SwdZhongxinghujiaCard,
                        crate::content::host::SWD_ZHONGXING_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("xingtigaizao") {
                    (
                        crate::content::host::HostBehavior::SwdXingtigaizaoCard,
                        crate::content::host::SWD_XINGTIGAIZAO_KEY,
                        Some(u.player),
                    )
                } else if name.contains("shijianhuisu") {
                    (
                        crate::content::host::HostBehavior::SwdShijianhuisuCard,
                        crate::content::host::SWD_SHIJIANHUISU_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("jinjilengdong") {
                    (
                        crate::content::host::HostBehavior::SwdJinjilengdongCard,
                        crate::content::host::SWD_JINJILENGDONG_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("shengxiuhaojiao") {
                    (
                        crate::content::host::HostBehavior::SwdShengxiuhaojiaoCard,
                        crate::content::host::SWD_SHENGXIUHAOJIAO_KEY,
                        u.targets.first().copied(),
                    )
                } else {
                    (
                        crate::content::host::HostBehavior::SwdYinmilichangCard,
                        crate::content::host::SWD_YINMILICHANG_KEY,
                        u.targets.first().copied(),
                    )
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: def_key.to_string(),
                    behavior,
                    owner: u.player,
                    target,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Changshezhen | CardKind::Pozhenjue => {
                // 长蛇阵/破阵诀（U05 zhenfa，规则 v201/T19b）：座位重排类
                //（swapSeat 原语 ADR 0781）；无懈窗口包装（dizaizhen 同款）
                let kind = ctx.catalog.card_kind(u.card);
                let (behavior, def_key) = if kind == CardKind::Changshezhen {
                    (
                        crate::content::host::HostBehavior::SwdChangshezhenCard,
                        crate::content::host::SWD_CHANGSHEZHEN_KEY,
                    )
                } else {
                    (
                        crate::content::host::HostBehavior::SwdPozhenjueCard,
                        crate::content::host::SWD_POZHENJUE_KEY,
                    )
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: def_key.to_string(),
                    behavior,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Tianfuzhen | CardKind::Qixingzhen | CardKind::Niaoxiangzhen => {
                // 阵法三张（U05 zhenfa，规则 v200/T19a）：AOE/自目标机器
                // + 无懈窗口包装（dizaizhen 同款）
                let kind = ctx.catalog.card_kind(u.card);
                let behavior = match kind {
                    CardKind::Tianfuzhen => crate::content::host::HostBehavior::SwdTianfuzhenCard,
                    CardKind::Qixingzhen => crate::content::host::HostBehavior::SwdQixingzhenCard,
                    _ => crate::content::host::HostBehavior::SwdNiaoxiangzhenCard,
                };
                let def_key = match kind {
                    CardKind::Tianfuzhen => crate::content::host::SWD_TIANFUZHEN_KEY,
                    CardKind::Qixingzhen => crate::content::host::SWD_QIXINGZHEN_KEY,
                    _ => crate::content::host::SWD_NIAOXIANGZHEN_KEY,
                };
                let mut phase2_queue: Vec<u32> = Vec::new();
                let targets_for_machine = match kind {
                    CardKind::Tianfuzhen => {
                        // AOE：全部大势力且有牌者（engine 校验同口径）
                        ctx.core
                            .alive_players()
                            .filter(|p| {
                                ctx.core.is_major_faction(ctx.catalog, *p)
                                    && (!ctx.core.player(*p).hand.is_empty()
                                        || !ctx.core.player(*p).equips.is_empty())
                            })
                            .collect()
                    }
                    CardKind::Qixingzhen => {
                        // 阶段一队列=全部围攻者（sieges(q, player)）；阶段二
                        // 队列同名单（存 cards，视为用杀）
                        let besiegers: Vec<u8> = ctx
                            .core
                            .alive_players()
                            .filter(|q| ctx.core.sieges(ctx.catalog, *q, u.player))
                            .collect();
                        phase2_queue = besiegers.iter().map(|q| u32::from(*q)).collect();
                        besiegers
                    }
                    _ => {
                        // 鸟翔阵 AOE：全部合法目标
                        ctx.core
                            .alive_players()
                            .filter(|t| {
                                *t != u.player
                                    && !ctx.core.is_friend_of(ctx.catalog, u.player, *t)
                                    && {
                                        let (n, pv) = ctx.core.alive_neighbors(*t);
                                        n.is_some_and(|q| {
                                            ctx.core.effective_faction(ctx.catalog, *t)
                                                == ctx.core.effective_faction(ctx.catalog, q)
                                        }) || pv.is_some_and(|q| {
                                            ctx.core.effective_faction(ctx.catalog, *t)
                                                == ctx.core.effective_faction(ctx.catalog, q)
                                        })
                                    }
                            })
                            .collect()
                    }
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: def_key.to_string(),
                    behavior,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: phase2_queue,
                    targets: targets_for_machine,
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Jiguan => {
                // 机关牌（swd 第十六/十七批，规则 v189/v190）：同 kind
                // 异行为按目录名分流（指路小虎=随机敌虚拟杀；木甲人=
                // 弃非基本换随机机关牌）——无懈窗口包装共享。
                let name = &ctx.catalog.cards[u.card as usize].name;
                let (behavior, def_key, target) = if name.contains("mujiaren") {
                    (
                        crate::content::host::HostBehavior::SwdMujiarenCard,
                        crate::content::host::SWD_MUJIAREN_KEY,
                        None,
                    )
                } else if name.contains("gouhunluo") {
                    (
                        crate::content::host::HostBehavior::SwdGouhunluoCard,
                        crate::content::host::SWD_GOUHUNLUO_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("jiguantong") {
                    (
                        crate::content::host::HostBehavior::SwdJiguantongCard,
                        crate::content::host::SWD_JIGUANTONG_KEY,
                        None,
                    )
                } else if name.contains("jiguanfeng") {
                    (
                        crate::content::host::HostBehavior::SwdJiguanfengCard,
                        crate::content::host::SWD_JIGUANFENG_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("jiguanyuan") {
                    (
                        crate::content::host::HostBehavior::SwdJiguanyuanCard,
                        crate::content::host::SWD_JIGUANYUAN_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("jiguanyaoshu") {
                    (
                        crate::content::host::HostBehavior::SwdJiguanyaoshuCard,
                        crate::content::host::SWD_JIGUANYAOSHU_KEY,
                        u.targets.first().copied(),
                    )
                } else if name.contains("jiguanshu") {
                    (
                        crate::content::host::HostBehavior::SwdJiguanshuCard,
                        crate::content::host::SWD_JIGUANSHU_KEY,
                        Some(u.player),
                    )
                } else {
                    (
                        crate::content::host::HostBehavior::SwdZhiluxiaohuCard,
                        crate::content::host::SWD_ZHILUXIAOHU_KEY,
                        None,
                    )
                };
                let targets_for_machine = if behavior
                    == crate::content::host::HostBehavior::SwdJiguantongCard
                {
                    // 逆序结算（规则 v193，上游 reverseOrder）：目标表
                    // 反转为结算序——从使用者上家逆座位序逐个结算
                    let mut ts = u.targets.clone();
                    ts.reverse();
                    ts
                } else {
                    Vec::new()
                };
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: def_key.to_string(),
                    behavior,
                    owner: u.player,
                    target,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: targets_for_machine,
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Yangpijuan => {
                // 羊皮卷（U05 swd 第十三批，规则 v97）：无懈窗口包装 +
                // 选类别 → 牌堆随机亮 3 张 → 选一张入手
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::SWD_YANGPIJUAN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::SwdYangpijuanCard,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Longlushanfeng => {
                // 龙颅山峰（U05 mtg 首切片，规则 v100）：换地 + 二选一
                // 获得杀/闪（上游 createCard 凭空造牌→原生牌堆取，登记
                // 差异）；地非锦囊，无无懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_LONGSHANFENG_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgLongshanfengLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Haidao => {
                // 海岛（U05 mtg 第二批，规则 v101）：地机器——满血加护
                // 甲/受伤弃牌回血或取消得护甲；地非锦囊，无无懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_HAIDAO_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgHaidaoLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Shuimomuxue => {
                // 水没墓穴（U05 mtg 第三批，规则 v102）：地机器——可弃
                // 一张摸二（可取消）；地非锦囊，无无懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_SHUIMOMUXUE_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgShuimomuxueLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Feixu => {
                // 废墟（U05 mtg 第四批，规则 v103）：地机器——从弃牌堆
                // 发现一张牌；地非锦囊，无无懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_FEIXU_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgFeixuLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Yixialan => {
                // 依夏兰（U05 mtg 第五批，规则 v104）：地机器——选类型
                // 随机获得基本/锦囊牌一张；地附主动技（重铸）经换地授予
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_YIXIALAN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgYixialanLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Shamolvzhou => {
                // 沙漠绿洲（U05 mtg 第六批，规则 v105）：地机器——从牌堆
                // 基本牌发现一张；地非锦囊，无无懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_SHAMOLVZHOU_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgShamolvzhouLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Linzhongjianta => {
                // 林中尖塔（U05 mtg 第七批，规则 v106）：地机器——发现
                // 武器并直接装备；地非锦囊，无无懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_LINZHONGJIANTA_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgLinzhongjiantaLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Bingheyaosai => {
                // 冰河要塞（U05 mtg 第八批，规则 v107）：地机器——摸二
                // 弃二（强制）；地非锦囊，无无懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_BINGHEYAOSAI_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgBingheyaosaiLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Youlin => {
                // 幽林（U05 mtg 第九批，规则 v108）：地机器——随机使用
                // 一张普通锦囊（池收窄登记见 catalog）；地非锦囊，无无
                // 懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_YOULIN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgYoulinLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Lindixiliu => {
                // 邻地溪流（U05 mtg 第十批，规则 v109）：地机器——♥入
                // 手 + 任意张♥弃 → 每张随机敌方置延时；地非锦囊，无无
                // 懈窗口
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::MTG_LINDIXILIU_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::MtgLindixiliuLand,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Dizaizhen => {
                // 地载阵（U05 zhenfa 首切片，规则 v110）：无懈窗口包装
                // + 所有非大势力存活角色各摸一张
                let effect = Box::new(DizaizhenMachine { owner: u.player });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Mianlijinzhen => {
                // 面里金针（G1 古剑，机关）：目标摸 1 然后失去 1 点体力
                //（规则 v77 对齐上游 loseHp；上游无 wuxieable，不包无懈窗口）
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::GJ_MIANLI_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::GujianMianliCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("mianlijinzhen has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(effect)
            }
            CardKind::Feibiao => {
                // 飞镖（G1 古剑，机关）：目标弃一张黑色手牌或受 1 点伤害
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::GJ_FEIBIAO_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::GujianFeibiaoCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("feibiao has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::SelectCard,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Chenhuodajie => {
                // 趁火打劫（G1 运筹帷幄，锦囊）：获得目标一张手牌
                //（登记差异：上游为目标任选可见牌，原生取首张确定性获得）
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::WD_CHENHUO_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::ChenhuodajieCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("chenhuodajie has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Fudichouxin => {
                // 釜底抽薪（G1 运筹帷幄，锦囊）：无懈窗口 + 拼点（两段选择）
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::WD_FUDI_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::FudichouxinCompare,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("fudichouxin has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::CompareTargetPick,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::WangmeiZhike => {
                // 望梅止渴（G1 玩点论杀，锦囊）：无懈窗口 + 目标最小体力/
                // 最小手牌条件增益（宿主原语）
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::WD_WANGMEIZHIKE_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::WangmeiZhikeCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("wangmeizhike has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::ShouLiJian => {
                // 手里剑（G1 欢乐卡牌）：目标弃 1 张装备或受 1 点伤害
                //（嵌套选择恢复样本，ADR 0776 规则宿主）
                Some(Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::HOST_SHOULIJIAN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::TargetDiscardEquipOrDamage,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("shoulijian has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::TargetChoice,
                }))
            }
            kind if kind.aoe_need().is_some() => {
                // AOE 的无懈窗口为**逐目标**（由 AoeResolveMachine 对每个目标
                // 自行开窗），外层不再包整牌级窗口——旧包裹会吞掉应答并把
                // 整张牌作废（本批修复，ADR 0705）。
                Some(Box::new(AoeResolveMachine {
                    card: u.card,
                    source: u.player,
                    need: kind.aoe_need().expect("checked"),
                    targets: u.targets,
                    idx: 0,
                    inner: None,
                }))
            }
            CardKind::Chain => {
                let effect = ChainMachine {
                    card: u.card,
                    source: u.player,
                    targets: u.targets,
                    emitted: 0,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            CardKind::Duel => {
                let effect = DuelMachine {
                    card: u.card,
                    source: u.player,
                    target: *u.targets.first().expect("duel has one target"),
                    on_target: true,
                    asked: false,
                    second_pending: false,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            CardKind::Zhufangshenshi => {
                // 贴房绅视（U05 swd 第十批，规则 v94）：他人锦囊——本回合
                // 无视距离指定该目标并摸一张牌。效果在 intrinsic 提交段
                // 同步执行（无需机器）；此处仅占位以透过穷尽 match。
                None
            }
            CardKind::Guiyoujie => {
                // 归有界（U05 swd，规则 v91）：他人延时锦囊——判定为黑色
                // 则失去 1 点体力 + 随机弃一张牌（上游 loseHp+randomDiscard）
                let target = *u.targets.first().expect("guiyoujie has one target");
                let effect = PlaceDelayedMachine {
                    card: u.card,
                    target,
                    placed: false,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            CardKind::Le | CardKind::Bing => {
                // 延时锦囊使用时即开无懈窗口（对齐旧线：牌先入桌面，
                // 无懈后入判定区、被无懈则弃置），窗口未抵消由
                // PlaceDelayedMachine 置入目标判定区。
                let target = *u.targets.first().expect("delayed trick has one target");
                let effect = PlaceDelayedMachine {
                    card: u.card,
                    target,
                    placed: false,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            CardKind::ShanDian => {
                let effect = PlaceDelayedMachine {
                    card: u.card,
                    target: u.player,
                    placed: false,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            CardKind::HongShui => {
                // 洪水：自-target 延时锦囊（判定：梅花 2–9 → 3 点雷伤）
                let effect = PlaceDelayedMachine {
                    card: u.card,
                    target: u.player,
                    placed: false,
                };
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, Box::new(effect))))
            }
            CardKind::Caochuanjiejian => {
                // 草船借箭（G1 运筹帷幄，锦囊）：无懈窗口 + 目标分支选择
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::WD_CAOCHUAN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::CaochuanCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("caochuan has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::SelectNature,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Geanguanhuo => {
                // 隔岸观火（G1 运筹帷幄，锦囊）：双目标拼点链
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::YC_GEAN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::GeanguanhuoCompare,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: u.targets.clone(),
                    stage: crate::content::host::HostStage::CompareTargetPick,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Linghunzhihuo => {
                // 灵魂之火（G1 炉石，锦囊）：目标 1 点火焰伤害，使用者弃一张手牌
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::HT_LINGHUN_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::LinghunzhihuoCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("linghunzhihuo has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Zhaomingdan => {
                // 照明弹（G1 炉石，锦囊）：弃目标一张手牌，双方各摸 1
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::HT_ZHAOMING_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::ZhaomingdanCard,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("zhaomingdan has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Jihuocard => {
                // 急耗卡（G1 炉石，锦囊）：自-target 摸 1（临时技暂缓）
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::HT_JIHUO_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::JihuocardDraw,
                    owner: u.player,
                    target: None,
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::Act,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            CardKind::Toulianghuanzhu => {
                // 偷梁换柱（G1 运筹帷幄，锦囊）：无懈窗口 + 换牌双选
                let effect = Box::new(crate::content::host::HostMachine {
                    def_key: crate::content::host::WD_TOULIANG_KEY.to_string(),
                    behavior: crate::content::host::HostBehavior::TouliangSwap,
                    owner: u.player,
                    target: Some(*u.targets.first().expect("touliang has one target")),
                    card: Some(u.card),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: crate::content::host::HostStage::SelectCard,
                });
                Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
            }
            _ => None,
            }
        }
        EventData::Phase { player, phase: Phase::Judge, begin: true } => {
            let judge = std::mem::take(&mut ctx.core.player_mut(player).judge);
            if judge.is_empty() {
                None
            } else {
                // 判定排序分组（N06-a，对齐旧线 pending_delayed_cards）：
                // 非闪电延时锦囊按放置序在前、闪电在后，队首先消费。
                let mut queue: Vec<crate::ids::CardId> = judge
                    .iter()
                    .copied()
                    .filter(|c| ctx.catalog.card_kind(*c) != CardKind::ShanDian)
                    .collect();
                queue.extend(
                    judge
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::ShanDian),
                );
                Some(Box::new(JudgePhaseMachine { owner: player, queue, current: None }))
            }
        }
        EventData::Phase { player, phase: Phase::Discard, begin: true } => {
            // 贪欲（v215）/庆祝（v245）：跳弃牌族——弃牌机器不生成（上游
            // phaseDiscardBefore trigger.cancel 同口径；此前贪欲面被测试
            // drain 掩盖，本臂补真实闸门）
            let mut skip_discard = [
                crate::content::standard::BOSS_TANYU,
                crate::content::standard::BOSS_QINGZHU,
            ]
            .iter()
            .any(|name| {
                ctx.catalog
                    .skill_by_name(name)
                    .map(|d| ctx.core.has_skill(player, d))
                    .unwrap_or(false)
            });
            // 猛态（Boss 年兽，规则 v248）：本回合未进入出牌阶段 → 跳过
            // 弃牌阶段（上游 boss_mengtai_discard trigger.cancel 同口径）
            if !skip_discard && ctx.core.turn.mengtai_use_pending {
                if let Some(mtdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_MENGTAI)
                {
                    if ctx.core.has_skill(player, mtdef) {
                        skip_discard = true;
                    }
                }
            }
            if skip_discard {
                ctx.core.turn.discard_phase_discarded = 0;
                None
            }
            // 克己（吕蒙）：本回合未使用杀则无需弃牌（手牌上限视为无限）
            else {
                let keji = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::KE_JI)
                    .map(|d| ctx.core.has_skill(player, d))
                    .unwrap_or(false);
                if keji && !ctx.core.turn.sha_used_play_phase {
                    None
                } else {
                let mut limit = ctx.core.hand_limit(ctx.catalog, player);
                // 机甲（U05 ow D.Va，规则 v141）：机甲血>0 时手牌上限 +4
                //（上游 mod maxHandcard storage.jijia 同口径）
                if ctx
                    .catalog
                    .skill_by_name(crate::content::host::WUDIAN_JIJIA_KEY)
                    .map(|def| ctx.core.has_skill(player, def))
                    .unwrap_or(false)
                    && ctx.core.player(player).jijia_hp > 0
                {
                    limit += 4;
                }
                // 蛊舞·缚（U05 xianjian 阿奴，规则 v154）：蛊舞生效中手牌
                // 上限 -1（上游 anwugu2 mod maxHandcard 同口径）
                if ctx.core.player(player).anwugu_count > 0 {
                    limit = limit.saturating_sub(1);
                }
                // 春饼（G1 古剑食物）：手牌上限 +1（持有状态即生效）
                if let Some(def) = ctx.catalog.skill_by_name(crate::content::host::GJ_FOOD_CHUNBING_KEY)
                {
                    if ctx
                        .core
                        .skills
                        .get(&(player, def))
                        .map(|i| i.enabled)
                        .unwrap_or(false)
                    {
                        limit += 1;
                    }
                }
                // 倾盆大雨（gwent 天气）：手牌上限 -1
                if let Some(def) = ctx.catalog.skill_by_name(crate::content::host::GW_QINPEN_STATE_KEY)
                {
                    if ctx
                        .core
                        .skills
                        .get(&(player, def))
                        .map(|i| i.enabled)
                        .unwrap_or(false)
                    {
                        limit = limit.saturating_sub(1);
                    }
                }
                // 妄尊印（袁术妄尊授予的回合级印记）：手牌上限 -1
                if let Some(def) = ctx.catalog.skill_by_name(crate::content::standard::WANG_ZUN_MARK) {
                    if ctx
                        .core
                        .skills
                        .get(&(player, def))
                        .map(|i| i.enabled)
                        .unwrap_or(false)
                    {
                        limit = limit.saturating_sub(1);
                    }
                }
                // 水没墓穴地图效果（mtg 第三批，规则 v102）：锁定全局——
                // 手牌上限 -1（current_land 状态直读，同倾盆大雨口径）
                if ctx.core.current_land == Some(crate::catalog::CardKind::Shuimomuxue) {
                    limit = limit.saturating_sub(1);
                }
                // 沙漠绿洲地图效果（mtg 第六批，规则 v105）：基本牌不
                // 计入手牌上限——需弃数按非基本手牌数计（上游
                // ignoredHandcard basic 同口径）
                let shamolvzhou =
                    ctx.core.current_land == Some(crate::catalog::CardKind::Shamolvzhou);
                // 红莲（Boss 毕方，规则 v228）：红色牌不计入手牌上限
                //（上游 ignoredHandcard red 同口径）
                let honglianx = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_HONGLIANX)
                    .map(|def| ctx.core.has_skill(player, def))
                    .unwrap_or(false);
                let hand = if shamolvzhou {
                    ctx.core
                        .player(player)
                        .hand
                        .iter()
                        .filter(|c| {
                            crate::content::host::yangpijuan_class(
                                ctx.catalog.card_kind(**c),
                            ) != Some(0)
                        })
                        .count()
                } else if honglianx {
                    ctx.core
                        .player(player)
                        .hand
                        .iter()
                        .filter(|c| {
                            !matches!(
                                ctx.catalog.cards[**c as usize].suit,
                                crate::catalog::Suit::Heart
                                    | crate::catalog::Suit::Diamond
                            )
                        })
                        .count()
                } else {
                    ctx.core.player(player).hand.len()
                };
                if hand > limit {
                    Some(Box::new(DiscardToLimitMachine {
                        player,
                        need: hand - limit,
                        asked: false,
                    }))
                } else {
                    None
                }
            }
        }
}
        EventData::Death { player, killer } => Some(Box::new(DeathMachine { player, killer })),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 内核机器
// ---------------------------------------------------------------------------

/// 杀的结算：询问目标闪，未闪则造成伤害子事件。
/// 裸衣伤害加成：来源本回合已发动裸衣（`once_used` 旗标，判定阶段
/// 开始时随 `reset_turn_limits` 清除）→ 杀/决斗伤害 +1。
fn luoyi_bonus(ctx: &Ctx<'_>, source: Option<PlayerId>) -> u32 {
    let Some(s) = source else { return 0 };
    let Some(def) = ctx
        .catalog
        .skill_by_name(crate::content::standard::LUO_YI)
    else {
        return 0;
    };
    match ctx.core.skills.get(&(s, def)) {
        Some(inst) if inst.enabled && inst.once_used => 1,
        _ => 0,
    }
}

/// 凶兽（Boss 模式共有技，规则 v208）：来源持凶兽且目标体力小于来源
/// 时，来源使用【杀】造成的伤害 +1（上游 boss_xiongshou_damage
/// `damageBegin1` filter hp< 同口径）。
fn xiongshou_bonus(ctx: &Ctx<'_>, source: PlayerId, target: PlayerId) -> u32 {
    let Some(def) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_XIONGSHOU)
    else {
        return 0;
    };
    let enabled = ctx
        .core
        .skills
        .get(&(source, def))
        .map(|i| i.enabled)
        .unwrap_or(false);
    if enabled && ctx.core.player(target).hp < ctx.core.player(source).hp {
        1
    } else {
        0
    }
}

/// 贪欲手牌「全场最多」判定：持有者手牌数不少于任何其他存活角色。
pub(crate) fn tanyu_is_max_hand(
    core: &crate::state::GameStateCore,
    holder: PlayerId,
) -> bool {
    let my = core.player(holder).hand.len();
    core.alive_players()
        .all(|o| o == holder || core.player(o).hand.len() <= my)
}

/// 积财（Boss 饕餮，规则 v216）：回复结算后——回复者==持有者 → 持有
/// 者摸 2；否则持有者与回复者各摸 1（上游 boss_jicai recoverAfter 同
/// 口径；仅实际回复量>0 的 Recover 事件进入）。
fn commit_recover_skills(ctx: &mut Ctx<'_>, recovered: PlayerId) {
    // 忍性（年兽形态，规则 v249）：global recoverEnd——持有者非当前回
    // 合角色时，任意角色回复后其摸 1（上游 boss_renxing recoverEnd 同口
    // 径；damageEnd 半面见 commit_damage 忍性臂）
    if let Some(rxdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_RENXING)
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
    let Some(jdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_JICAI)
    else {
        return;
    };
    let holders: Vec<PlayerId> = ctx
        .core
        .skills
        .iter()
        .filter(|((_, dd), i)| *dd == jdef && i.enabled)
        .map(|((o, _), _)| *o)
        .filter(|o| ctx.core.player(*o).alive)
        .collect();
    for holder in holders {
        if recovered == holder {
            ctx.draw_cards(holder, 2);
        } else {
            ctx.draw_cards(holder, 1);
            if ctx.core.player(recovered).alive {
                ctx.draw_cards(recovered, 1);
            }
        }
    }
    // 伤逝（规则 v260）：体力变化（回复）后补牌
    shangshix_draw(ctx, recovered);
}

/// 覆泰2（Boss 穷奇，规则 v214）：场上存有持覆泰的穷奇、用/救桃者非
/// 穷奇且当前回合非穷奇时，桃不可用/不可救（上游 boss_futai2
/// cardSavable/cardEnabled 双 mod 同口径）。
pub(crate) fn tao_futai2_banned(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    user: PlayerId,
) -> bool {
    let Some(fdef) = catalog.skill_by_name(crate::content::standard::BOSS_FUTAI) else {
        return false;
    };
    let holders: Vec<PlayerId> = core
        .skills
        .iter()
        .filter(|((_, dd), i)| *dd == fdef && i.enabled)
        .map(|((o, _), _)| *o)
        .filter(|o| core.player(*o).alive && *o != user)
        .collect();
    match holders.first() {
        Some(&boss) => core.turn.current != boss,
        None => false,
    }
}

/// P1/N08 急救（华佗）：持有者濒死结算时红色手牌可当【桃】使用。
fn is_tao_like(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    card: crate::ids::CardId,
    dying: PlayerId,
) -> bool {
    if ctx.catalog.card_kind(card) == CardKind::Tao {
        // 覆泰2（规则 v214）：全场禁桃时真桃不可救
        return !tao_futai2_banned(ctx.core, ctx.catalog, responder);
    }
    // 酒（G1 cardpile 前置）：濒死时对自身使用回复 1 点体力
    if ctx.catalog.card_kind(card) == CardKind::Jiu {
        return true;
    }
    // 护符（U05 swd 首切片，规则 v85）：仅濒死者本人的桃形救援
    //（上游 savable: dying === player）
    if ctx.catalog.card_kind(card) == CardKind::Hufu {
        return responder == dying;
    }
    // 天仙酒（U05 swd 第五批，规则 v89）：酒的 swd 变体——濒死自救
    if ctx.catalog.card_kind(card) == CardKind::Tianxianjiu {
        return true;
    }
    // 急救（v202 对齐上游 viewAsFilter）：限持有者回合外——自己是当前
    // 回合角色时不可发动
    let jijiu = ctx
        .catalog
        .skill_by_name(crate::content::standard::JI_JIU)
        .map(|d| ctx.core.has_skill(responder, d) && ctx.core.turn.current != responder)
        .unwrap_or(false);
    if jijiu
        && matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    {
        return true;
    }
    // 复燃（Boss 焰灵，规则 v241）：焰灵濒死时其敌方可用红色牌当桃救
    //（上游 boss_furan2 dying chooseToUse viewAs tao 同口径）
    let furan = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_FURAN)
        .map(|d| ctx.core.has_skill(dying, d))
        .unwrap_or(false);
    if furan
        && ctx.core.is_boss_side(responder) != ctx.core.is_boss_side(dying)
        && matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    {
        return true;
    }
    // 蜜链藕（G1 古剑食物）：持有者红桃手牌可当桃（濒死响应）
    let mizhilianou = ctx
        .catalog
        .skill_by_name(crate::content::host::GJ_FOOD_MIZHLIANOU_KEY)
        .map(|d| ctx.core.has_skill(responder, d))
        .unwrap_or(false);
    if mizhilianou && ctx.catalog.cards[card as usize].suit == crate::catalog::Suit::Heart {
        return true;
    }
    // 帝女玄霜（G1 古剑，濒死可救基本牌）：按桃结算回复 1 体力
    //（弃摸骑乘由 DyingSaveMachine rider 承担）
    ctx.catalog.card_kind(card) == CardKind::Dinvxuanshuang
}

#[derive(Clone)]
pub struct ShaResolveMachine {
    card: crate::ids::CardId,
    source: PlayerId,
    target: PlayerId,
    nature: crate::event::DamageNature,
    asked: bool,
    /// 八卦阵判定是否已做（每次杀结算至多一次）。
    armor_judged: bool,
    /// 已使用的闪数（无双：需要连续两张）。
    dodges_used: u8,
    /// 铁骑判定结果：None=未判定/无铁骑；Some(true)=红色（不能出闪）。
    tieji_red: Option<bool>,
    /// 流离（大乔）状态（入快照，ADR 0774：消除恢复分歧）。
    liuli_done: bool,
    /// 流离两段询问（逐技对照批二 v76）：1=弃牌选择挂起，2=转移目标挂起。
    liuli_stage: u8,
    /// 流离已选定的待弃牌（stage 2 期间暂存）。
    liuli_card: Option<crate::ids::CardId>,
    /// 护驾（曹操）逐人询问队列（入快照）：`None` = 本响应窗口尚未
    /// 开始逐人询问；`Some(q)` = 剩余待问的魏势力角色（目标下家起
    /// 按座位序）；`Some(空)` = 已全部问完（拒绝/无人合格）。
    /// 不以暗手牌预筛。
    hujia_queue: Option<Vec<PlayerId>>,
    /// 护驾当前被询问的持牌者（Some = 等待其 Confirm/Pass）。
    hujia_holder: Option<PlayerId>,
    /// 酒效果（派发时从回合状态捕获）：本次杀伤害 +1。
    jiu_bonus: bool,
    /// 麒麟弓（批③-b 第二把）：弃马询问挂起中。
    asking_qilin: bool,
    qilin_horse: Option<crate::ids::CardId>,
    /// 寒冰剑（批③-b 第三把）：防止伤害确认挂起中。
    asking_hanbing: bool,
    /// 风雪刃（U05 swd 第十一批，规则 v95）：命中后翻面+摸牌询问挂起中。
    asking_fengxueren: bool,
    /// 贯石斧（批③-b 第五把）：杀被闪后弃二强伤询问挂起中
    ///（Some = 待弃的牌；None = 询问未开）。
    asking_guanshi: bool,
    guanshi_cards: Vec<crate::ids::CardId>,
    /// 青龙偃月刀（批③-b 第六把）：被闪后「再杀」询问挂起中
    ///（Some = 选中的新杀牌，来自来源手牌）。
    asking_qinglong: bool,
    /// 雌雄双股剑（批③-b 第八把）：弃牌/令摸询问挂起中。
    asking_cixiong: bool,
    /// 雌雄双股剑：本杀已问过标志。
    cixiong_done: bool,
    /// 寒冰剑（逐技对照批一，规则 v75）：目标自选弃牌剩余次数
    ///（来源确认防止后为 2，逐次扣减；上游逐张强制弃）。
    hanbing_pending: u8,
    /// 方天画戟（批③-b 第九把）：当前目标之后的剩余目标队列。
    extra_targets: Vec<PlayerId>,
    /// 霰弹（U05 ow 死神，规则 v140）：弃牌询问挂起中。
    asking_xiandan: bool,
    /// 霰弹：本杀已问过标志（每杀至多一次）。
    xiandan_done: bool,
    /// 霰弹：黑牌分支——本杀伤害 +1（上游 xiandan2 damageBegin num++
    /// 同口径，一次性随本杀）。
    xiandan_bonus: bool,
    /// 掠影（U05 xianjian 慕长兰，规则 v154）：目标选人询问挂起中。
    asking_lueying: bool,
    /// 掠影：目标已选定的受害者（等来源随机弃其一张）。
    lueying_victim: Option<PlayerId>,
    /// 掠影：本杀已询问且被放弃（v194——放弃不消耗 usable，但同一杀
    /// 不得重复询问；阻止 step 重入钩子再问）。
    lueying_declined: bool,
}

impl ShaResolveMachine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        target: PlayerId,
        nature: crate::event::DamageNature,
        asked: bool,
        armor_judged: bool,
        dodges_used: u8,
        tieji_red: Option<bool>,
        liuli_done: bool,
        liuli_stage: u8,
        liuli_card: Option<crate::ids::CardId>,
        hujia_queue: Option<Vec<PlayerId>>,
        hujia_holder: Option<PlayerId>,
        jiu_bonus: bool,
        asking_qilin: bool,
        qilin_horse: Option<crate::ids::CardId>,
        asking_hanbing: bool,
        asking_fengxueren: bool,
        asking_guanshi: bool,
        guanshi_cards: Vec<crate::ids::CardId>,
        asking_qinglong: bool,
        asking_cixiong: bool,
        cixiong_done: bool,
        extra_targets: Vec<PlayerId>,
        hanbing_pending: u8,
        asking_xiandan: bool,
        xiandan_done: bool,
        xiandan_bonus: bool,
        asking_lueying: bool,
        lueying_victim: Option<PlayerId>,
        lueying_declined: bool,
    ) -> Self {
        Self {
            card,
            source,
            target,
            nature,
            asked,
            armor_judged,
            dodges_used,
            tieji_red,
            liuli_done,
            liuli_stage,
            liuli_card,
            hujia_queue,
            hujia_holder,
            jiu_bonus,
            asking_qilin,
            qilin_horse,
            asking_hanbing,
            asking_fengxueren,
            asking_guanshi,
            guanshi_cards,
            asking_qinglong,
            asking_cixiong,
            cixiong_done,
            extra_targets,
            hanbing_pending,
            asking_xiandan,
            xiandan_done,
            xiandan_bonus,
            asking_lueying,
            lueying_victim,
            lueying_declined,
        }
    }

    /// 流离转移候选：存活、非自己、非此杀使用者、在（被转移者）攻击
    /// 范围内且为该杀合法目标（空城不可）——上游 filter/filterTarget 同口径。
    fn liuli_candidates(&self, ctx: &Ctx<'_>) -> Vec<ChoiceOption> {
        ctx.core
            .alive_players()
            .filter(|p| self.liuli_candidate_ok(ctx, *p))
            .map(ChoiceOption::Player)
            .collect()
    }

    fn liuli_candidate_ok(&self, ctx: &Ctx<'_>, p: PlayerId) -> bool {
        if p == self.target || p == self.source || !ctx.core.player(p).alive {
            return false;
        }
        let kongcheng = ctx
            .catalog
            .skill_by_name(crate::content::standard::KONG_CHENG)
            .map(|d| ctx.core.has_skill(p, d))
            .unwrap_or(false);
        if kongcheng && ctx.core.player(p).hand.is_empty() {
            return false;
        }
        ctx.core.computed_distance(ctx.catalog, self.target, p)
            <= ctx.core.attack_range(ctx.catalog, self.target)
    }

    /// 方天画戟（批③-b 第九把）：当前目标结算完毕 → 队列取下一目标
    ///（逐目标状态复位，重开该目标的防具/出闪询问）；队列空 → 弃置杀
    /// 本体结束。
    fn advance_target(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if let Some(next) = self.extra_targets.first().copied() {
            self.extra_targets.remove(0);
            self.target = next;
            self.asked = false;
            self.armor_judged = false;
            self.dodges_used = 0;
            self.tieji_red = None;
            self.liuli_done = false;
            self.liuli_stage = 0;
            self.liuli_card = None;
            self.hujia_queue = None;
            self.hujia_holder = None;
            self.asking_hanbing = false;
            self.asking_qilin = false;
            self.asking_guanshi = false;
            self.asking_fengxueren = false;
            self.hanbing_pending = 0;
            return Flow::Yield;
        }
        let card = self.card;
        ctx.to_discard(card);
        ctx.core.table.retain(|c| *c != card);
        Flow::Done
    }

    /// 结算杀伤害（裸衣/酒/凶兽加成）并推进目标/弃置杀本体。
    fn emit_sha_damage(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if ctx.core.player(self.target).alive {
            ctx.emit(EventData::Damage(damage_data_with(
                Some(self.source),
                self.target,
                1 + luoyi_bonus(ctx, Some(self.source))
                    + xiongshou_bonus(ctx, self.source, self.target)
                    + self.jiu_damage_bonus()
                    + u32::from(self.xiandan_bonus),
                self.nature,
                Some(self.card),
            )));
        }
        self.xiandan_bonus = false;
        self.advance_target(ctx)
    }

    /// 来源是否持有无双（需要连续两张闪）。
    fn source_has_wushuang(&self, ctx: &Ctx<'_>) -> bool {
        ctx.catalog
            .skill_by_name(crate::content::standard::WU_SHUANG)
            .map(|d| ctx.core.has_skill(self.source, d))
            .unwrap_or(false)
    }

    /// 酒伤害加成（派发时已捕获）。
    fn jiu_damage_bonus(&self) -> u32 {
        if self.jiu_bonus { 1 } else { 0 }
    }

    /// 护驾是否可发动：目标持技 + 主公资格（身份局须主公）。
    fn hujia_active(&self, ctx: &Ctx<'_>) -> bool {
        let hujia = ctx
            .catalog
            .skill_by_name(crate::content::standard::HU_JIA)
            .map(|d| ctx.core.has_skill(self.target, d))
            .unwrap_or(false);
        hujia && ctx.core.is_lord(self.target)
    }

    /// 护驾询问队列：目标下家起按座位序的魏势力存活角色
    /// （不以暗手牌预筛——询问资格公开，应答私密）。
    fn build_hujia_queue(&self, ctx: &Ctx<'_>) -> Vec<PlayerId> {
        ctx.core
            .seat_order_from(ctx.core.next_alive(self.target))
            .into_iter()
            .filter(|p| {
                *p != self.target
                    && ctx.core.player(*p).alive
                    && ctx.core.effective_faction(ctx.catalog, *p) == Some(crate::catalog::Faction::Wei)
            })
            .collect()
    }
}

impl ShaResolveMachine {
    /// 堡垒哨卫（U05 ow，规则 v125）：来源持 bshaowei 且哨卫模式——
    /// 杀无视距离/防具、无次数限制、不可闪避。
    fn baolei_sentinel(&self, ctx: &Ctx<'_>) -> bool {
        ctx.catalog
            .skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY)
            .map(|def| {
                ctx.core.has_skill(self.source, def)
                    && ctx.core.player(self.source).baolei_mode == 1
            })
            .unwrap_or(false)
    }

    /// 寒冰剑可发动（来源装备寒冰剑且目标有手牌/装备）。
    fn hanbing_can_fire(&self, ctx: &Ctx<'_>) -> bool {
        let has_hanbing = ctx
            .core
            .player(self.source)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Hanbing);
        has_hanbing
            && ctx.core.player(self.target).alive
            && (!ctx.core.player(self.target).hand.is_empty()
                || !ctx.core.player(self.target).equips.is_empty())
    }

    /// 目标「手牌+装备」是否仍有可弃牌（上游 hasDiscardableCards("he")）。
    fn target_has_he(&self, ctx: &Ctx<'_>) -> bool {
        !ctx.core.player(self.target).hand.is_empty()
            || !ctx.core.player(self.target).equips.is_empty()
    }

    /// 寒冰剑目标自选弃牌询问：手牌+装备逐张强选（min=1 不可取消）。
    fn open_hanbing_target_ask(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let mut options: Vec<ChoiceOption> = ctx
            .core
            .player(self.target)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        options.extend(
            ctx.core
                .player(self.target)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card),
        );
        Flow::WaitChoice {
            observer: self.target,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "hanbing_target_discard",
                options,
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }

    /// 麒麟弓可弃目标坐骑列表（来源装备麒麟弓且目标有马）。
    fn qilin_horses(&self, ctx: &Ctx<'_>) -> Vec<ChoiceOption> {
        let has_qilin = ctx
            .core
            .player(self.source)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qilin);
        if !has_qilin {
            return Vec::new();
        }
        ctx.core.player(self.target)
            .equips
            .iter()
            .filter(|c| {
                matches!(
                    ctx.catalog.card_kind(**c),
                    CardKind::HorsePlus | CardKind::HorseMinus
                )
            })
            .copied()
            .map(ChoiceOption::Card)
            .collect()
    }

}

impl crate::catalog::SkillMachine for ShaResolveMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Sha {
            card: self.card,
            source: self.source,
            target: self.target,
            nature: self.nature,
            asked: self.asked,
            armor_judged: self.armor_judged,
            dodges_used: self.dodges_used,
            tieji_red: self.tieji_red,
            liuli_done: self.liuli_done,
            liuli_stage: self.liuli_stage,
            liuli_card: self.liuli_card.map(|c| c as u32),
            hujia_queue: self.hujia_queue.clone(),
            hujia_holder: self.hujia_holder,
            jiu_bonus: self.jiu_bonus,
            asking_qilin: self.asking_qilin,
            qilin_horse: self.qilin_horse,
            asking_hanbing: self.asking_hanbing,
            asking_fengxueren: self.asking_fengxueren,
            asking_guanshi: self.asking_guanshi,
            guanshi_cards: self.guanshi_cards.iter().copied().collect(),
            asking_qinglong: self.asking_qinglong,
            asking_cixiong: self.asking_cixiong,
            cixiong_done: self.cixiong_done,
            extra_targets: self.extra_targets.iter().copied().map(|c| c as u32).collect(),
            hanbing_pending: self.hanbing_pending,
            asking_xiandan: self.asking_xiandan,
            xiandan_done: self.xiandan_done,
            xiandan_bonus: self.xiandan_bonus,
            asking_lueying: self.asking_lueying,
            lueying_victim: self.lueying_victim.map(|c| c as u32),
            lueying_declined: self.lueying_declined,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 寒冰剑（逐技对照批一，规则 v75）：目标自选弃牌恢复——上游
        // discardPlayerCard("he", forced) 逐张强制弃，目标从手牌+装备
        // 自选；次数用尽或目标无牌可弃后跳过伤害。
        // 霰弹（U05 ow 死神，规则 v140）：弃牌应答——红=直击（哨卫同
        // 款直中路径）；黑=本杀伤害 +1；取消=照常进入防具/闪询问。
        if self.asking_xiandan {
            self.asking_xiandan = false;
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                let red = matches!(
                    ctx.catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                let in_hand = ctx.core.player(self.source).hand.contains(&c);
                if in_hand {
                    ctx.discard_from_hand(self.source, c);
                } else if ctx.core.player(self.source).equips.contains(&c) {
                    ctx.core.player_mut(self.source).equips.retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                ctx.log(LogEvent::CardDiscarded {
                    player: self.source,
                    cards: vec![c],
                });
                if red {
                    if ctx.core.player(self.target).alive {
                        return self.emit_sha_damage(ctx);
                    }
                    return self.advance_target(ctx);
                }
                self.xiandan_bonus = true;
            }
            // 黑弃/取消：复位 asked 落回 !asked 分支重开出闪询问（青龙
            // 再杀 reboot 同款——否则霰弹应答会被门控应答区误读）。
            self.asked = false;
        }
        // 掠影（规则 v154；v194 两相协议）：相以应答形状区分——确认相
        // 只会收到 Confirm（确认）或空（放弃），选人相只会收到 Player。
        // Confirm=消耗 usable、获得目标 he 随机一张，仍有其他有牌者则
        // 转入目标选人相；空=放弃（不消耗 usable，重开闪询问）；Player=
        // 受害者选定，来源随机弃其 he 一张后复位 asked。
        if self.asking_lueying {
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(v)) => {
                    self.asking_lueying = false;
                    self.lueying_victim = Some(v);
                    let pool: Vec<crate::ids::CardId> = {
                        let p = ctx.core.player(v);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool[k];
                        ctx.take_owned_card(v, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![dc],
                        });
                    }
                    self.asked = false;
                }
                Some(ChoiceOption::Confirm) => {
                    ctx.core.turn.lueying_used = true;
                    // 获得目标 he 随机一张
                    let pool: Vec<crate::ids::CardId> = {
                        let p = ctx.core.player(self.target);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let card = pool[k];
                        ctx.take_owned_card(self.target, card);
                        ctx.core.player_mut(self.source).hand.push(card);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.target,
                            to: self.source,
                            cards: vec![card],
                        });
                    }
                    // 仍有其他有牌者→目标选人相；否则本技结束
                    let others: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.source
                                && *p != self.target
                                && (!ctx.core.player(*p).hand.is_empty()
                                    || !ctx.core.player(*p).equips.is_empty())
                        })
                        .collect();
                    if others.is_empty() {
                        self.asking_lueying = false;
                        self.asked = false;
                    } else {
                        let options: Vec<ChoiceOption> = others
                            .into_iter()
                            .map(ChoiceOption::Player)
                            .collect();
                        return Flow::WaitChoice {
                            observer: self.target,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: "lueying_choose_victim",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                }
                _ => {
                    // 放弃：不消耗 usable（同上游 usable 口径），标记本杀
                    // 已问过（阻止 step 重入再问），重开闪询问
                    self.asking_lueying = false;
                    self.lueying_declined = true;
                    self.asked = false;
                }
            }
        }
        if self.hanbing_pending > 0 {
            let pick = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            self.hanbing_pending -= 1;
            if let Some(c) = pick {
                if ctx.core.player(self.target).hand.contains(&c) {
                    ctx.discard_from_hand(self.target, c);
                } else if ctx.core.player(self.target).equips.contains(&c) {
                    ctx.core
                        .player_mut(self.target)
                        .equips
                        .retain(|x| *x != c);
                    crate::kernel::revoke_equip_skills(
                        ctx.core,
                        ctx.catalog,
                        self.target,
                        c,
                    );
                    ctx.to_discard(c);
                }
            }
            if self.hanbing_pending > 0 && self.target_has_he(ctx) {
                return self.open_hanbing_target_ask(ctx);
            }
            self.hanbing_pending = 0;
            return self.advance_target(ctx);
        }
        // 寒冰剑（批③-b 第三把）：来源防止伤害确认恢复——确认后转入
        // 目标自选弃牌（两次；逐技对照批一消除「自动依次」差异）。
        // 风雪刃恢复：确认→目标翻面+摸 1（无伤害）；放弃→照常伤害
        if self.asking_fengxueren {
            self.asking_fengxueren = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed {
                let flipped = {
                    let t = ctx.core.player_mut(self.target);
                    t.face_down = !t.face_down;
                    t.face_down
                };
                ctx.emit(EventData::TurnOvered {
                    player: self.target,
                    face_down: flipped,
                });
                ctx.draw_cards(self.target, 1);
                return self.advance_target(ctx);
            }
            return self.emit_sha_damage(ctx);
        }
        if self.asking_hanbing {
            self.asking_hanbing = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed {
                self.hanbing_pending = 2;
                if self.target_has_he(ctx) {
                    return self.open_hanbing_target_ask(ctx);
                }
                self.hanbing_pending = 0;
                return self.advance_target(ctx);
            }
            // 放弃：照常进入麒麟弓/伤害结算（单武器互斥，实际直接伤害）。
            return self.emit_sha_damage(ctx);
        }
        // 贯石斧（批③-b 第五把）：弃二询问恢复——按应答弃牌，强制伤害。
        if self.asking_guanshi {
            self.asking_guanshi = false;
            let picks: Vec<crate::ids::CardId> = ctx
                .answer()
                .iter()
                .filter_map(|c| match c {
                    ChoiceOption::Card(card) => Some(*card),
                    _ => None,
                })
                .collect();
            let fired = picks.len() == 2;
            for card in picks {
                let in_hand = ctx.core.player(self.source).hand.contains(&card);
                if in_hand {
                    ctx.discard_from_hand(self.source, card);
                } else if ctx.core.player(self.source).equips.contains(&card) {
                    ctx.core.player_mut(self.source).equips.retain(|x| *x != card);
                    ctx.to_discard(card);
                }
            }
            self.guanshi_cards.clear();
            // 恰弃两张 → 强制伤害（被闪仍命中）；放弃 → 无伤。
            if fired && ctx.core.player(self.target).alive {
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(self.source),
                    self.target,
                    1,
                    self.nature,
                    Some(self.card),
                )));
            }
            return self.advance_target(ctx);
        }
        // 青龙偃月刀（批③-b 第六把）：再杀询问恢复——弃置选中的杀并
        // 以其重启对同一目标的杀结算（防/闪/贯石斧/麒麟弓流程全部重走）。
        if self.asking_qinglong {
            self.asking_qinglong = false;
            if let Some(ChoiceOption::Card(new_sha)) = ctx.answer().first().copied() {
                if ctx.core.player(self.source).hand.contains(&new_sha) {
                    ctx.discard_from_hand(self.source, new_sha);
                    ctx.emit(EventData::CardResponded {
                        card: new_sha,
                        player: self.source,
                    });
                    self.card = new_sha;
                    self.asked = false;
                    self.armor_judged = false;
                    self.dodges_used = 0;
                    self.tieji_red = None;
                    self.liuli_done = false;
                    self.liuli_stage = 0;
                    self.liuli_card = None;
                    return Flow::Yield;
                }
            }
            // 放弃：当前目标按被闪结算（无伤）→ 推进队列/结束。
            return self.advance_target(ctx);
        }
        // 雌雄双股剑（批③-b 第八把；逐技对照批一 v75 消除首张差异）：
        // 弃牌/令摸恢复——上游 chooseToDiscard 一张手牌、可取消（取消=
        // 令来源摸 1），随后照常进入防具/出闪流程。
        if self.asking_cixiong {
            self.asking_cixiong = false;
            self.asked = false;
            let pick = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            if let Some(c) = pick.filter(|c| ctx.core.player(self.target).hand.contains(c)) {
                ctx.discard_from_hand(self.target, c);
            } else if ctx.core.player(self.source).alive {
                ctx.draw_cards(self.source, 1);
            }
            // 落入下方 !asked 分支重开出闪询问（防具判定已由
            // armor_judged 标记幂等）。
        }
        // 麒麟弓（批③-b 第二把）：弃马询问恢复——按应答弃马，随后结算伤害。
        if self.asking_qilin {
            self.asking_qilin = false;
            // 空应答 = 来源放弃弃马。
            if let Some(ChoiceOption::Card(horse)) = ctx.answer().first().copied() {
                if ctx.core.player(self.target).equips.contains(&horse) {
                    ctx.core.player_mut(self.target).equips.retain(|x| *x != horse);
                    ctx.to_discard(horse);
                }
            }
            self.qilin_horse = None;
            self.emit_sha_damage(ctx);
            return Flow::Done;
        }
        // 青釭剑（P2/N08 批③-b）：锁定——无视目标防具（来源装备青釭时
        // 八卦阵判定与仁王盾免疫均不生效）。
        let qinggang_pierce = ctx
            .core
            .player(self.source)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qinggang);
        // 八卦阵：受击者装备防具时，出闪询问前先做防具判定（每杀至多一次）
        // ——红桃视为已出闪（伤害免除），否则照常进入出闪询问。
        // 焚天（Boss 朱雀，规则 v236）：红色杀 directHit 无视防具
        //（上游 boss_fentian2 directHit 全场同口径——原生防具=八卦阵）
        let fentian_direct_hit = {
            let red = matches!(
                ctx.catalog.cards[self.card as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            red
                && ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_FENTIAN)
                    .map(|def| ctx.core.has_skill(self.source, def))
                    .unwrap_or(false)
        };
        if !self.armor_judged {
            self.armor_judged = true;
            let has_armor = !qinggang_pierce
                && !fentian_direct_hit
                && ctx.core.player(self.target).alive
                && ctx
                    .core
                    .player(self.target)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::Armor);
            if has_armor {
                let judged = ctx.draw_judge_card();
                let success = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Heart
                );
                ctx.log(LogEvent::ArmorJudged {
                    player: self.target,
                    card: judged,
                    success,
                });
                // 判定牌判定后即弃置（八卦阵判定无获得路径）
                ctx.to_discard(judged);
                if success {
                    let card = self.card;
                    ctx.to_discard(card);
                    ctx.core.table.retain(|c| *c != card);
                    return Flow::Done;
                }
            }
        }
        // 流离（大乔；逐技对照批二 v76 对齐上游 chooseCardTarget）：成为
        // 杀目标时可自选弃置一张牌（手牌+装备）并将此杀转移给攻击范围内
        // 的一名其他角色（非此杀使用者、非自己、且为该杀合法目标——空城
        // 不可）。整体可取消：弃牌询问空应答=不发动；先选牌后选目标，
        // 转移生效后弃置暂存牌（上游 content 顺序）。
        if !self.liuli_done {
            self.liuli_done = true;
            let liuli = ctx
                .catalog
                .skill_by_name(crate::content::standard::LIU_LI)
                .map(|d| ctx.core.has_skill(self.target, d))
                .unwrap_or(false);
            let has_he = !ctx.core.player(self.target).hand.is_empty()
                || !ctx.core.player(self.target).equips.is_empty();
            if liuli && has_he && !self.liuli_candidates(ctx).is_empty() {
                let mut options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.target)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    ctx.core
                        .player(self.target)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card),
                );
                self.liuli_stage = 1;
                return Flow::WaitChoice {
                    observer: self.target,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "liuli_discard_choose",
                        options,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
        } else if self.liuli_stage == 1 {
            self.liuli_stage = 0;
            let pick = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            if let Some(c) = pick.filter(|c| {
                ctx.core.player(self.target).hand.contains(c)
                    || ctx.core.player(self.target).equips.contains(c)
            }) {
                self.liuli_card = Some(c);
                let candidates = self.liuli_candidates(ctx);
                if !candidates.is_empty() {
                    self.liuli_stage = 2;
                    return Flow::WaitChoice {
                        observer: self.target,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "liuli_transfer",
                            options: candidates,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    };
                }
                self.liuli_card = None;
            }
        } else if self.liuli_stage == 2 {
            self.liuli_stage = 0;
            let from = self.target;
            if let Some(ChoiceOption::Player(new_target)) = ctx.answer().first().copied() {
                if self.liuli_candidate_ok(ctx, new_target) {
                    self.target = new_target;
                    // 新目标重新判定八卦/铁骑；护驾窗口对新目标重开
                    self.armor_judged = false;
                    self.tieji_red = None;
                    self.asked = false;
                    self.hujia_queue = None;
                    self.hujia_holder = None;
                }
            }
            // 上游 content：转移生效后弃置所选牌（手牌或装备；牌在
            // 被转移者 from 的区域——self.target 已指向新目标）。
            if let Some(c) = self.liuli_card.take() {
                let in_hand = ctx.core.player(from).hand.contains(&c);
                if in_hand {
                    ctx.discard_from_hand(from, c);
                } else if ctx.core.player(from).equips.contains(&c) {
                    ctx.core.player_mut(from).equips.retain(|x| *x != c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, c);
                    ctx.to_discard(c);
                }
                ctx.log(LogEvent::CardDiscarded {
                    player: from,
                    cards: vec![c],
                });
            }
        }
        // 护驾（曹操主公技，ADR 0776 重构对齐上游）：目标需出闪时，
        // 先按座位顺序逐人询问魏势力角色「是否替其打出一张闪」——
        // 不以暗手牌预筛（询问资格公开、应答私密，不向攻击者泄漏
        // 谁持闪）；拒绝继续问下一位，任一人确认即代出一张闪
        // （计入无双需求），全部拒绝后回到目标本人询问。
        let needed: u8 = if self.source_has_wushuang(ctx) { 2 } else { 1 };
        if self.dodges_used >= needed {
            let card = self.card;
            ctx.to_discard(card);
            ctx.core.table.retain(|c| *c != card);
            return Flow::Done;
        }
        if self.hujia_active(ctx) {
            // 本响应窗口首次进入：构建逐人询问队列
            if self.hujia_queue.is_none() {
                self.hujia_queue = Some(self.build_hujia_queue(ctx));
            }
            // 消费当前被询问者应答（若有）
            if let Some(holder) = self.hujia_holder.take() {
                // v204 对齐上游 chooseToRespond：持牌者自选一张闪打出
                //（含倾国转化：黑手牌可代出）；取消/空选 = 拒绝，问下一位
                if let Some(ChoiceOption::Card(shan)) = ctx.answer().first().copied() {
                    if ctx.core.player(holder).hand.contains(&shan) {
                        ctx.discard_from_hand(holder, shan);
                        ctx.emit(EventData::CardResponded {
                            card: shan,
                            player: holder,
                        });
                        self.dodges_used += 1;
                        if self.dodges_used >= needed {
                            let card = self.card;
                            ctx.to_discard(card);
                            ctx.core.table.retain(|c| *c != card);
                            return Flow::Done;
                        }
                        // 无双还需第二张：新响应窗口——重开逐人询问
                        //（含目标本人的再次询问）
                        self.hujia_queue = Some(self.build_hujia_queue(ctx));
                        self.asked = false;
                        return Flow::Yield;
                    }
                    // 请求期间牌已不在手（并发消耗防御）：视为拒绝
                }
            }
            // 逐人询问下一位（Pass 后继续）
            let queue = self.hujia_queue.as_mut().expect("queue built above");
            if let Some(pos) = queue
                .iter()
                .position(|p| ctx.core.player(*p).alive)
            {
                let next = queue.remove(pos);
                self.hujia_holder = Some(next);
                // 候选=自己手牌中的真闪 + 倾国转化（黑手牌）；不含他人牌
                let qingguo_next = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::QING_GUO)
                    .map(|d| ctx.core.has_skill(next, d))
                    .unwrap_or(false);
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(next)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        let kind = ctx.catalog.card_kind(*c);
                        kind == CardKind::Shan
                            || (qingguo_next
                                && matches!(
                                    ctx.catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                ))
                    })
                    .map(|c| ChoiceOption::Card(c))
                    .collect();
                return Flow::WaitChoice {
                    observer: next,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "hujia_respond",
                        options,
                        min: 0,
                        max: 1,
                        allow_cancel: false, // 空选=拒绝本位继续问下一位（非取消）
                    },
                };
            }
            // 队列问完（全部拒绝或无人合格）：Some(空) 固定，
            // 进入目标本人询问；后续恢复重入不再重建。
        }
        if !self.asked {
            self.asked = true;
            // 天弦（U05 xianjian 柳梦璃，规则 v184）：锁定——杀对每目标
            // 结算前目标判定（上游 useCardToBefore 同窗口，priority 5.5
            // 先于结算技），黑色则取消对该目标的结算（不进入闪询问；每
            // 目标一次=asked 每目标复位）。**登记差异**：同铁骑内联判定
            // 口径=确定性判定，无鬼道/天妒改判窗。
            {
                let tianxian = ctx
                    .catalog
                    .skill_by_name(crate::content::host::WUDIAN_XJ_TIANXIAN_KEY)
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                if tianxian {
                    let judged = ctx.draw_judge_card();
                    let black = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    ctx.to_discard(judged);
                    if black {
                        return self.advance_target(ctx);
                    }
                }
            }
            // 铁骑（马超）：锁定式判定——判定牌为红色则目标不能出闪响应
            let tieji = ctx
                .catalog
                .skill_by_name(crate::content::standard::TIE_JI)
                .map(|d| ctx.core.has_skill(self.source, d))
                .unwrap_or(false);
            if tieji && self.tieji_red.is_none() {
                let judged = ctx.draw_judge_card();
                let red = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                ctx.to_discard(judged);
                self.tieji_red = Some(red);
            }
            // 仁王盾（P2/N08 批③-a）：锁定——黑色【杀】对装备者无效
            //（先于八卦阵判定；红杀/无盾照常进入判定与出闪询问）。
            let renwang_blocks = !qinggang_pierce && {
                let black = matches!(
                    ctx.catalog.cards[self.card as usize].suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                black
                    && ctx
                        .core
                        .player(self.target)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Renwang)
            };
            if renwang_blocks {
                let card = self.card;
                ctx.to_discard(card);
                ctx.core.table.retain(|c| *c != card);
                return Flow::Done;
            }
            // 堡垒哨卫（U05 ow，规则 v125）：哨卫模式的杀不可闪避/无视
            // 防具——直接结算伤害（上游 bshaowei_sha directHit 同口径）
            if self.baolei_sentinel(ctx) && ctx.core.player(self.target).alive {
                return self.emit_sha_damage(ctx);
            }
            // 霰弹（U05 ow 死神，规则 v140）：shaBegin direct——来源可弃
            // 1 张 he：红=直击（跳过闪询问）；黑=本杀伤害 +1；取消=照常
            //（he 空不询问；每杀至多一次——上游 shaBegin 每杀事件同口径）。
            // 龙翔（U05 xianjian 温慧，规则 v146）：shaBegin forced——
            // 目标手牌多于自己时随机弃差值张（上游 randomGets(diff) 同
            // 口径；无询问零字段）。
            {
                let has_longxiang = ctx
                    .catalog
                    .skill_by_name(crate::content::host::WUDIAN_LONGXIANG_KEY)
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                if has_longxiang
                    && ctx.core.player(self.target).alive
                    && ctx.core.player(self.target).hand.len()
                        > ctx.core.player(self.source).hand.len()
                {
                    let diff = ctx.core.player(self.target).hand.len()
                        - ctx.core.player(self.source).hand.len();
                    let hs: Vec<crate::ids::CardId> = ctx
                        .core
                        .player(self.target)
                        .hand
                        .iter()
                        .copied()
                        .collect();
                    let mut pool = hs;
                    for _ in 0..diff {
                        if pool.is_empty() {
                            break;
                        }
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool.swap_remove(k);
                        let t = self.target;
                        ctx.take_owned_card(t, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![dc],
                        });
                    }
                }
            }
            // 掠影（U05 xianjian 慕长兰，规则 v154；v194 对齐上游）：
            // shaBegin usable:1，非强制——filter 要求目标 he 有牌**且**
            // 另有一名其他有牌角色（缺第三者则整个技能不可发动，连获得
            // 也不触发）。确认后获得目标 he 随机一张；仍有其他有牌者→
            // 目标选人、来源随机弃其一张（lueying_used 确认后消耗，放弃
            // 不计数，同上游 usable 口径）。
            {
                let has_lueying = ctx
                    .catalog
                    .skill_by_name(crate::content::host::WUDIAN_LUEYING_KEY)
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                let t_he = !ctx.core.player(self.target).hand.is_empty()
                    || !ctx.core.player(self.target).equips.is_empty();
                let others: Vec<PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.source
                            && *p != self.target
                            && (!ctx.core.player(*p).hand.is_empty()
                                || !ctx.core.player(*p).equips.is_empty())
                    })
                    .collect();
                if has_lueying
                    && !ctx.core.turn.lueying_used
                    && !self.lueying_declined
                    && ctx.core.player(self.source).alive
                    && ctx.core.player(self.target).alive
                    && t_he
                    && !others.is_empty()
                {
                    // 确认相：复用 asking_lueying 挂起（恢复侧按应答形状
                    // 区分相——Confirm=确认发动，Player=受害者已选定）
                    self.asking_lueying = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "lueying_confirm",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
            }
            let has_xiandan = ctx
                .catalog
                .skill_by_name(crate::content::host::WUDIAN_XIANDAN_KEY)
                .map(|d| ctx.core.has_skill(self.source, d))
                .unwrap_or(false);
            if has_xiandan
                && !self.xiandan_done
                && ctx.core.player(self.source).alive
                && ctx.core.player(self.target).alive
                && (!ctx.core.player(self.source).hand.is_empty()
                    || !ctx.core.player(self.source).equips.is_empty())
            {
                self.xiandan_done = true;
                self.asking_xiandan = true;
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.source)
                    .hand
                    .iter()
                    .copied()
                    .chain(ctx.core.player(self.source).equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "xiandan_discard",
                        options,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            // 雌雄双股剑（批③-b 第八把）：杀指定异性目标后（性别有槽取
            // 武将性别、无槽默认男），目标选择：弃一张手牌（自选，逐技
            // 对照批一 v75 对齐上游 chooseToDiscard）或取消令使用者摸 1；
            // 无手牌则仅可令摸（上游同：!hasCards("h") 直接 draw）。
            if !self.cixiong_done
                && !self.asking_cixiong
                && ctx
                    .core
                    .player(self.source)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::Cixiong)
                && ctx.core.player(self.target).alive
                && ctx.core.effective_gender(self.source)
                    != ctx.core.effective_gender(self.target)
            {
                self.cixiong_done = true;
                let has_hand = !ctx.core.player(self.target).hand.is_empty();
                if has_hand {
                    self.asking_cixiong = true;
                    // 上游 chooseToDiscard：目标自选一张手牌，空应答（取消）
                    // = 令使用者摸 1（逐技对照批一 v75）。
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.target)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    return Flow::WaitChoice {
                        observer: self.target,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "cixiong_discard_choose",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                // 无手牌：唯一可选 = 令使用者摸 1。
                ctx.draw_cards(self.source, 1);
            }
            // 龙胆：杀可当闪打出——响应选项并入目标手牌中的杀
            let longdan = ctx
                .catalog
                .skill_by_name(crate::content::standard::LONG_DAN)
                .map(|d| ctx.core.has_skill(self.target, d))
                .unwrap_or(false);
            let tieji_blocks = self.tieji_red == Some(true);
            // 倾国（甄姬，v202 对齐上游 position "hs"）：黑色手牌可当闪打出——并入响应选项
            let qingguo = ctx
                .catalog
                .skill_by_name(crate::content::standard::QING_GUO)
                .map(|d| ctx.core.has_skill(self.target, d))
                .unwrap_or(false);
            // 堡垒哨卫（U05 ow，规则 v125）目标侧：持有者不能闪避杀
            let target_sentinel = ctx
                .catalog
                .skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY)
                .map(|def| {
                    ctx.core.has_skill(self.target, def)
                        && ctx.core.player(self.target).baolei_mode == 1
                })
                .unwrap_or(false);
            let mut shans: Vec<ChoiceOption> = ctx.core.player(self.target)
                .hand
                .iter()
                .filter(|c| {
                    !tieji_blocks
                        && !target_sentinel
                        && {
                            let kind = ctx.catalog.card_kind(**c);
                            kind == CardKind::Shan || (longdan && kind == CardKind::Sha)
                        }
                })
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            // 诱敌深入（G1 玩点论杀）：打出后杀无效，来源受 1 点反噬
            //（登记差异：上游为强制来源连用杀，原生简化为单次反噬）
            for c in ctx.core.player(self.target).hand.iter().copied() {
                if ctx.catalog.card_kind(c) == CardKind::Youdishenru {
                    shans.push(ChoiceOption::Card(c));
                    break;
                }
            }
            if qingguo && !tieji_blocks {
                for c in ctx.core.player(self.target).hand.iter().copied() {
                    let kind = ctx.catalog.card_kind(c);
                    let already = kind == CardKind::Shan || (longdan && kind == CardKind::Sha);
                    let black = matches!(
                        ctx.catalog.cards[c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    if black && !already {
                        shans.push(ChoiceOption::Card(c));
                    }
                }
            }

            if shans.is_empty() || !ctx.core.player(self.target).alive {
                // 风雪刃（U05 swd 第十一批，规则 v95）：命中后可选择——
                // 防止伤害，目标翻面并摸一张牌（上游 shaHit → unhurt +
                // turnOver + draw；Confirm 可放弃则照常伤害）
                if !self.asking_fengxueren
                    && ctx.core.player(self.target).alive
                    && !ctx.core.player(self.target).face_down
                    && ctx
                        .core
                        .player(self.source)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Fengxueren)
                {
                    self.asking_fengxueren = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "fengxueren_flip_confirm",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                // 寒冰剑：目标有牌时来源可防止伤害，改为目标自选弃两张
                //（逐技对照批一 v75 对齐上游 discardPlayerCard）。
                if self.hanbing_can_fire(ctx) {
                    self.asking_hanbing = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "hanbing_prevent_confirm",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                // 麒麟弓：伤害结算前来源可弃目标一匹坐骑。
                let horses = self.qilin_horses(ctx);
                if !horses.is_empty() && ctx.core.player(self.target).alive {
                    self.asking_qilin = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "qilin_discard_horse",
                            options: horses,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                return self.emit_sha_damage(ctx);
            }
            // 司圃（Boss 应昭，规则 v232）：应昭出牌阶段已用牌数≤2 期间
            // 其他人无法打出牌（上游 cardRespondable 同口径）——出闪选项
            // 仅保留 Pass（目标必受伤害）
            let sipu_ban_respond = ctx
                .catalog
                .skill_by_name(crate::content::standard::BOSS_SIPU)
                .map(|def| {
                    ctx.core.has_skill(self.source, def)
                        && ctx.core.turn.current == self.source
                        && ctx.core.turn.cards_used_play_phase < 3
                })
                .unwrap_or(false);
            let mut options = vec![ChoiceOption::Pass];
            if !sipu_ban_respond {
                options.extend(shans);
            }
            return Flow::WaitChoice {
                observer: self.target,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "respond_shan",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
        // 已恢复：读取应答
        let answer = ctx.answer_first();
        // 诱敌深入应答：杀无效（弃置），来源受 1 点反噬伤害
        if let ChoiceOption::Card(yd) = answer {
            if ctx.catalog.card_kind(yd) == CardKind::Youdishenru {
                let in_hand = ctx.core.player(self.target).hand.contains(&yd);
                if in_hand {
                    ctx.discard_from_hand(self.target, yd);
                }
                ctx.emit(EventData::CardResponded {
                    card: yd,
                    player: self.target,
                });
                if ctx.core.player(self.source).alive {
                    ctx.emit(EventData::Damage(damage_data_with(
                        Some(self.target),
                        self.source,
                        1,
                        crate::event::DamageNature::Normal,
                        Some(yd),
                    )));
                }
                let card = self.card;
                ctx.to_discard(card);
                ctx.core.table.retain(|c| *c != card);
                return Flow::Done;
            }
        }
        if let ChoiceOption::Card(shan) = answer {
            // 倾国应答可能来自装备区：按实际所在区移除
            let in_hand = ctx.core.player(self.target).hand.contains(&shan);
            if in_hand {
                ctx.discard_from_hand(self.target, shan);
            } else {
                ctx.core
                    .player_mut(self.target)
                    .equips
                    .retain(|x| *x != shan);
                ctx.to_discard(shan);
            }
            ctx.emit(EventData::CardResponded {
                card: shan,
                player: self.target,
            });
            // 雷击（Boss 青龙，规则 v234）：响应闪后判定雷击（无双第二
            // 闪重入本臂=每张闪各触发一次，上游同口径）
            releiji_on_shan(ctx, self.target);
            self.dodges_used += 1;
            // 无双：还需第二张闪 → 新响应窗口（重开护驾逐人询问 +
            // 目标本人询问；不弃杀、不受伤）
            let needed: u8 = if self.source_has_wushuang(ctx) { 2 } else { 1 };
            if self.dodges_used < needed {
                self.hujia_queue = None;
                self.hujia_holder = None;
                self.asked = false;
                return Flow::Yield;
            }
            // 青龙偃月刀（批③-b 第六把）：杀被闪后，来源可立即对相同
            // 目标再使用一张【杀】（从手牌选择；放弃则不再追问——每杀
            // 至多追问一次，防止连环杀循环）。
            let has_qinglong = ctx
                .core
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qinglong);
            let hand_shas: Vec<ChoiceOption> = ctx
                .core
                .player(self.source)
                .hand
                .iter()
                .filter(|c| {
                    matches!(
                        ctx.catalog.card_kind(**c),
                        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
                    )
                })
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if has_qinglong && !hand_shas.is_empty() && !self.asking_qinglong {
                self.asking_qinglong = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "qinglong_replay_sha",
                        options: hand_shas,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            // 贯石斧（批③-b 第五把）：杀被闪后，来源可弃两张牌强制伤害
            //（登记差异：可弃牌池不含贯石斧本体；不足两张则无法发动）。
            let has_guanshi = ctx
                .core
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Guanshi);
            if has_guanshi && ctx.core.player(self.source).alive {
                let mut pool: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.source)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                pool.extend(
                    ctx.core
                        .player(self.source)
                        .equips
                        .iter()
                        .filter(|c| ctx.catalog.card_kind(**c) != CardKind::Guanshi)
                        .copied()
                        .map(ChoiceOption::Card),
                );
                if pool.len() >= 2 {
                    self.asking_guanshi = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "guanshi_force_damage",
                            options: pool,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    };
                }
            }
        } else if ctx.core.player(self.target).alive {
            if self.hanbing_can_fire(ctx) {
                self.asking_hanbing = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::Confirm,
                        prompt: "hanbing_prevent_confirm",
                        options: vec![ChoiceOption::Confirm],
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            let horses = self.qilin_horses(ctx);
            if !horses.is_empty() {
                self.asking_qilin = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "qilin_discard_horse",
                        options: horses,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            return self.emit_sha_damage(ctx);
        }
        // 目标已死等无伤害路径：推进队列/弃置杀本体。
        return self.advance_target(ctx);
    }
}

/// 濒死救援：按座位顺序询问持桃者；无人救则进入死亡子事件。
#[derive(Clone)]
pub struct DyingSaveMachine {
    dying: PlayerId,
    source: Option<PlayerId>,
    responders: Vec<PlayerId>,
    asked: bool,
    /// 本轮是否已有人用桃（整轮无人用桃 → 濒死结算死亡，不再循环询问）。
    round_used: bool,
}

impl DyingSaveMachine {
    pub(crate) fn new(
        dying: PlayerId,
        source: Option<PlayerId>,
        responders: Vec<PlayerId>,
        asked: bool,
        round_used: bool,
    ) -> Self {
        Self { dying, source, responders, asked, round_used }
    }
}

impl crate::catalog::SkillMachine for DyingSaveMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::DyingSave {
            dying: self.dying,
            source: self.source,
            responders: self.responders.clone(),
            asked: self.asked,
            round_used: self.round_used,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 刺猬（U07 玩点论杀韩龙，规则 v111）：濒死即死标记在位——跳
        // 过救援询问直接死亡（上游 ciqiu_dying dyingBegin firstDo die
        // 同口径）
        if ctx.core.turn.ciqiu_dying.contains(&self.dying) {
            ctx.core.turn.ciqiu_dying.retain(|p| *p != self.dying);
            ctx.emit(EventData::Death {
                player: self.dying,
                killer: self.source,
            });
            return Flow::Done;
        }
        loop {
            let p = ctx.core.player(self.dying);
            if p.hp > 0 && !p.dying {
                // 救援成功（dyingAfter 等价，规则 v157）：发射救回事件，
                // 觉醒技（姜城·焚逝）等触发挂在该事件的 AfterTriggers 窗
                // 口——时序在固有救援机之后，与上游 dyingAfter 一致。
                ctx.emit(EventData::DyingSaved {
                    player: self.dying,
                    source: self.source,
                });
                return Flow::Done; // 已被救回
            }
            // 跳过已死亡或无桃的应答者；队列耗尽即一轮问完——
            // 本轮有人用桃且场上仍有桃 → 开新一轮（负体力可由同一
            // 持桃者跨轮连续多张救回）；否则无人愿救，落至死亡。
            loop {
                while let Some(r) = self.responders.first() {
                    let alive = ctx.core.player(*r).alive;
                    let has_tao =
                        alive && ctx.core.player(*r).hand.iter().any(|c| {
                            is_tao_like(ctx, *r, *c, self.dying)
                        });
                    if !has_tao {
                        self.responders.remove(0);
                    } else {
                        break;
                    }
                }
                if !self.responders.is_empty() || !self.round_used {
                    break;
                }
                let any_tao = ctx.core.players.iter().enumerate().any(|(i, p)| {
                    p.alive && p.hand.iter().any(|c| is_tao_like(ctx, i as PlayerId, *c, self.dying))
                });
                if !any_tao {
                    break;
                }
                self.responders = ctx.core.seat_order_from(self.dying);
                self.round_used = false;
            }
            let Some(responder) = self.responders.first().copied() else {
                ctx.emit(EventData::Death {
                    player: self.dying,
                    killer: self.source,
                });
                return Flow::Done;
            };
            if self.asked {
                self.asked = false;
                self.responders.remove(0);
                let answer = ctx.answer().clone();
                if let Some(ChoiceOption::Card(tao)) = answer.first().copied() {
                    ctx.discard_from_hand(responder, tao);
                    // 救援（孙权主公技，锁定；v202 对齐上游 filter）：主公
                    // 资格 + 用桃者为吴势力 + 非濒死者本人——桃回复 +1
                    let jiuyuan = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::JIU_YUAN)
                        .map(|d| {
                            ctx.core.has_skill(self.dying, d)
                                && responder != self.dying
                                && ctx.core.is_lord(self.dying)
                                && ctx.core.effective_faction(ctx.catalog, responder)
                                    == Some(crate::catalog::Faction::Wu)
                        })
                        .unwrap_or(false);
                    let amount = if jiuyuan { 2 } else { 1 };
                    ctx.recover(self.dying, amount, Some(tao));
                    ctx.emit(EventData::CardResponded {
                        card: tao,
                        player: responder,
                    });
                    ctx.log(LogEvent::DyingSaved {
                        player: self.dying,
                        by_card: tao,
                    });
                    self.round_used = true;
                }
                continue; // 无论用桃或放弃，继续处理队列（轮转见队首过滤）
            }
            self.asked = true;
            let mut options = vec![ChoiceOption::Pass];
            // 司圃（Boss 应昭，规则 v223/v232）：应昭出牌阶段已用牌数≤2
            // 期间其他人不可救（上游 cardSavable 同口径）——非应昭救援者
            // 的桃不入选项（酒同理由 cardRespondable）
            let sipu_ban_save = ctx
                .catalog
                .skill_by_name(crate::content::standard::BOSS_SIPU)
                .map(|def| {
                    ctx.core.has_skill(self.dying, def)
                        && ctx.core.turn.current == self.dying
                        && ctx.core.turn.cards_used_play_phase < 3
                })
                .unwrap_or(false);
            let sipu_holder = if ctx
                .catalog
                .skill_by_name(crate::content::standard::BOSS_SIPU)
                .is_some()
            {
                self.dying
            } else {
                0
            };
            options.extend(
                ctx.core.player(responder)
                    .hand
                    .iter()
                    .filter(|c| is_tao_like(ctx, responder, **c, self.dying))
                    .filter(|c| {
                        !sipu_ban_save
                            || (responder == sipu_holder
                                && is_tao_like(ctx, responder, **c, self.dying))
                    })
                    .copied()
                    .map(ChoiceOption::Card),
            );
            return Flow::WaitChoice {
                observer: responder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "dying_save",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
    }
}

/// 决斗：目标先出杀，双方交替出杀；轮到某方其无杀（不询问）或放弃时，
/// 该方受对方造成的 1 点伤害，决斗结束。响应杀不占出牌阶段限一次；
/// 无距离限制——轮转起手/跳过语义/来源归属见覆盖清单行为差异表。
#[derive(Clone)]
pub struct DuelMachine {
    card: crate::ids::CardId,
    source: PlayerId,
    target: PlayerId,
    /// true = 轮到目标出杀；false = 轮到来源出杀。
    on_target: bool,
    asked: bool,
    /// 无双：对手需再次出杀（本轮已出一张）。
    second_pending: bool,
}

impl DuelMachine {
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        target: PlayerId,
        on_target: bool,
        asked: bool,
        second_pending: bool,
    ) -> Self {
        Self { card, source, target, on_target, asked, second_pending }
    }
}

impl crate::catalog::SkillMachine for DuelMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Duel {
            card: self.card,
            source: self.source,
            target: self.target,
            on_target: self.on_target,
            asked: self.asked,
            second_pending: self.second_pending,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            let responder = if self.on_target {
                self.target
            } else {
                self.source
            };
            let opponent = if self.on_target {
                self.source
            } else {
                self.target
            };
            let shas: Vec<crate::ids::CardId> = ctx.core.player(responder)
                .hand
                .iter()
                .copied()
                .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Sha)
                .collect();
            if !self.asked {
                if shas.is_empty() || !ctx.core.player(responder).alive {
                    // 轮到方无杀：不询问，直接受伤，决斗结束
                    ctx.emit(EventData::Damage(damage_data(
                        Some(opponent),
                        responder,
                        1 + luoyi_bonus(ctx, Some(opponent)),
                    )));
                    return self.finish(ctx);
                }
                self.asked = true;
                let mut options = vec![ChoiceOption::Pass];
                options.extend(shas.into_iter().map(ChoiceOption::Card));
                return Flow::WaitChoice {
                    observer: responder,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::RespondCard,
                        prompt: "duel_sha",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                };
            }
            // 应答消费
            self.asked = false;
            let answer = ctx.answer().clone();
            if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                ctx.discard_from_hand(responder, c);
                ctx.emit(EventData::CardResponded {
                    card: c,
                    player: responder,
                });
                // 无双：与无双持有者决斗的角色每次需连续打出两张杀——
                // 第一张后不轮转，同一响应者再次询问（second_pending）
                let wushuang_holder = if self.on_target { self.source } else { self.target };
                let opponent_has_wushuang = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::WU_SHUANG)
                    .map(|d| ctx.core.has_skill(wushuang_holder, d))
                    .unwrap_or(false);
                if opponent_has_wushuang && !self.second_pending {
                    self.second_pending = true;
                    continue; // 同一响应者再次询问第二张杀
                }
                self.second_pending = false;
                self.on_target = !self.on_target;
                continue;
            }
            // 放弃 → 受伤，决斗结束（重置无双标记）
            self.second_pending = false;
            ctx.emit(EventData::Damage(damage_data(
                Some(opponent),
                responder,
                1 + luoyi_bonus(ctx, Some(opponent)),
            )));
            return self.finish(ctx);
        }
    }
}

impl DuelMachine {
    fn finish(&self, ctx: &mut Ctx<'_>) -> Flow {
        let card = self.card;
        ctx.to_discard(card);
        ctx.core.table.retain(|c| *c != card);
        Flow::Done
    }
}

/// 无中生有结算：使用者摸 2。
#[derive(Clone)]
pub struct WuzhongMachine {
    pub owner: PlayerId,
    pub card: Option<crate::ids::CardId>,
}

impl crate::catalog::SkillMachine for WuzhongMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Wuzhong { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 2);
        // 弃置锦囊本体（按牌种扫表：恢复期安全，不依赖快照携带 card）。
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Wuzhong)
        {
            let card = ctx.core.table.remove(pos);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 桃园结义结算：所有其他存活角色回复 1 点体力。
#[derive(Clone)]
pub struct TaoyuanMachine {
    pub owner: PlayerId,
    pub card: Option<crate::ids::CardId>,
}

impl crate::catalog::SkillMachine for TaoyuanMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Taoyuan { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let others: Vec<PlayerId> = ctx
            .core
            .alive_players()
            .filter(|p| *p != self.owner)
            .collect();
        for p in others {
            let (hp, max) = {
                let pl = ctx.core.player(p);
                (pl.hp, pl.max_hp)
            };
            if hp < max as i32 {
                ctx.recover(p, 1, None);
            }
        }
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Taoyuan)
        {
            let card = ctx.core.table.remove(pos);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 地载阵结算（zhenfa 首切片，规则 v110）：所有非大势力存活角色各摸
/// 一张（上游 target.draw(false) 同口径；大势力=is_major_faction）。
#[derive(Clone)]
pub struct DizaizhenMachine {
    pub owner: PlayerId,
}

impl crate::catalog::SkillMachine for DizaizhenMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        // 无询问机器（逐目标摸牌，Noop 同款快照形态）
        crate::snap::MachineSnapshot::Noop
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let targets: Vec<PlayerId> = ctx
            .core
            .alive_players()
            .filter(|p| !ctx.core.is_major_faction(ctx.catalog, *p))
            .collect();
        for t in targets {
            ctx.draw_cards(t, 1);
        }
        // 本体牌离结算区入弃堆（Wugu 同款收尾）
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == CardKind::Dizaizhen)
        {
            let c = ctx.core.table.remove(pos);
            ctx.to_discard(c);
        }
        Flow::Done
    }
}

/// 五谷丰登结算：亮 min(存活数, 牌堆) 张，从使用者起按座位序逐人择一。
#[derive(Clone)]
pub struct WuguMachine {
    pub owner: PlayerId,
    pub cards: Vec<crate::ids::CardId>,
    pub pickers: Vec<PlayerId>,
    pub stage: WuguStage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WuguStage {
    /// 亮牌并确定择序（首个询问）。
    Init,
    /// 逐人读应答/开下一询问。
    Pick,
}

impl crate::catalog::SkillMachine for WuguMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Wugu {
            owner: self.owner,
            cards: self.cards.iter().copied().map(|c| c as u32).collect(),
            pickers: self.pickers.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            WuguStage::Init => {
                let alive: Vec<PlayerId> = ctx.core.alive_players().collect();
                let take = alive.len().min(ctx.core.deck.len());
                if take == 0 || self.cards.is_empty() == false {
                    return Flow::Done;
                }
                let new_len = ctx.core.deck.len() - take;
                self.cards = ctx.core.deck.split_off(new_len);
                self.pickers = alive;
                self.stage = WuguStage::Pick;
                self.open_pick(ctx)
            }
            WuguStage::Pick => {
                // 读当前择牌者应答，交牌并推进。
                let picker = match self.pickers.first().copied() {
                    Some(p) => p,
                    None => return self.finish_wugu(ctx),
                };
                if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
                    if let Some(pos) = self.cards.iter().position(|c| *c == card) {
                        self.cards.remove(pos);
                        if ctx.core.player(picker).alive {
                            ctx.core.player_mut(picker).hand.push(card);
                            ctx.out.push(crate::LogEvent::CardsDrawn {
                                player: picker,
                                cards: vec![card],
                            });
                        }
                    }
                }
                self.pickers.remove(0);
                if self.pickers.is_empty() || self.cards.is_empty() {
                    return self.finish_wugu(ctx);
                }
                self.open_pick(ctx)
            }
        }
    }
}

impl WuguMachine {
    fn open_pick(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(picker) = self.pickers.first().copied() else {
            return self.finish_wugu(ctx);
        };
        if !ctx.core.player(picker).alive || self.cards.is_empty() {
            self.pickers.remove(0);
            if self.pickers.is_empty() {
                return self.finish_wugu(ctx);
            }
            return self.open_pick(ctx);
        }
        let options: Vec<ChoiceOption> =
            self.cards.iter().copied().map(ChoiceOption::Card).collect();
        Flow::WaitChoice {
            observer: picker,
            spec: crate::choice::ChoiceSpec {
                kind: crate::choice::ChoiceKind::SelectCards,
                prompt: "wugu_pick",
                options,
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }
    fn finish_wugu(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        for c in self.cards.drain(..) {
            ctx.to_discard(c);
        }
        if let Some(c) = ctx.core.table.iter().position(|c| {
            ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Wugu
        }) {
            let card = ctx.core.table.remove(c);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 借刀杀人结算：武器持有者出杀（登记差异：直接伤害，无出闪窗口）
/// 或把武器交給另一名目标。
#[derive(Clone)]
pub struct JiedaoMachine {
    pub owner: PlayerId,
    pub holder: PlayerId,
    pub victim: PlayerId,
    pub card: Option<crate::ids::CardId>,
    pub stage: JiedaoStage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JiedaoStage {
    Ask,
    Read,
}

impl crate::catalog::SkillMachine for JiedaoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Jiedao {
            owner: self.owner,
            holder: self.holder,
            victim: self.victim,
            stage: match self.stage {
                JiedaoStage::Ask => crate::snap::JiedaoStageSnap::Ask,
                JiedaoStage::Read => crate::snap::JiedaoStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JiedaoStage::Ask => {
                // 资格复核：持有者仍在且武器仍在。
                let has_weapon = ctx
                    .core
                    .player(self.holder)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Weapon);
                if !ctx.core.player(self.holder).alive || !has_weapon {
                    return self.finish_jiedao(ctx);
                }
                self.stage = JiedaoStage::Read;
                Flow::WaitChoice {
                    observer: self.holder,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::PickTag,
                        prompt: "jiedao_sha_or_give",
                        options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            JiedaoStage::Read => {
                match ctx.answer_first() {
                    // 出杀：直接伤害结算（登记差异：无出闪窗口）。
                    ChoiceOption::Tag(0) => {
                        if ctx.core.player(self.victim).alive {
                            ctx.emit(crate::event::EventData::Damage(
                                crate::machine::damage_data(
                                    Some(self.holder),
                                    self.victim,
                                    1,
                                ),
                            ));
                        }
                    }
                    // 交出武器：武器移入另一目标装备区（原武器若有则替换
                    // 入弃牌堆）。
                    _ => {
                        let weapon = ctx
                            .core
                            .player(self.holder)
                            .equips
                            .iter()
                            .copied()
                            .find(|c| {
                                ctx.catalog.card_kind(*c)
                                    == crate::catalog::CardKind::Weapon
                            });
                        if let Some(w) = weapon {
                            ctx.core.player_mut(self.holder).equips.retain(|c| *c != w);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.holder,
                                w,
                            );
                            let victim = self.victim;
                            let old = ctx
                                .core
                                .player_mut(victim)
                                .equips
                                .iter()
                                .copied()
                                .find(|c| {
                                    ctx.catalog.card_kind(*c)
                                        == crate::catalog::CardKind::Weapon
                                });
                            if let Some(old_w) = old {
                                ctx.core.player_mut(victim).equips.retain(|c| *c != old_w);
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    victim,
                                    old_w,
                                );
                                ctx.to_discard(old_w);
                            }
                            ctx.core.player_mut(victim).equips.push(w);
                        }
                    }
                }
                self.finish_jiedao(ctx)
            }
        }
    }
}

impl JiedaoMachine {
    fn finish_jiedao(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        self.card = None;
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Jiedao)
        {
            let card = ctx.core.table.remove(pos);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 过河拆桥：弃置目标角色一张牌（手牌优先，其次装备区，再次判定区）。
#[derive(Clone)]
pub struct DismantleMachine {
    pub(crate) card: crate::ids::CardId,
    pub(crate) source: PlayerId,
    pub(crate) target: PlayerId,
    pub(crate) asked: bool,
}

impl crate::catalog::SkillMachine for DismantleMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Dismantle {
            card: self.card,
            source: self.source,
            target: self.target,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            // 选牌：手牌优先，其次装备区，再次判定区
            let t = ctx.core.player(self.target);
            let pick = t
                .hand
                .first()
                .copied()
                .or_else(|| t.equips.first().copied())
                .or_else(|| t.judge.first().copied());
            if let Some(c) = pick {
                // 从所在区移除
                let from_hand = t.hand.contains(&c);
                {
                    let was_equip = ctx.catalog.card_kind(c).is_equip();
                    let p = ctx.core.player_mut(self.target);
                    p.hand.retain(|x| *x != c);
                    p.equips.retain(|x| *x != c);
                    p.judge.retain(|x| *x != c);
                    if was_equip {
                        revoke_equip_skills(ctx.core, ctx.catalog, self.target, c);
                    }
                }
                ctx.to_discard(c);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.target,
                    cards: vec![c],
                });
                if from_hand {
                    // 弃置事件：连营类「失去手牌」触发依赖此事件（ADR 0763）
                    ctx.emit(EventData::CardsDiscarded {
                        player: self.target,
                        cards: vec![c],
                    });
                }
            }
            let card = self.card;
            ctx.to_discard(card);
            ctx.core.table.retain(|x| *x != card);
        }
        Flow::Done
    }
}

/// 顺手牵羊结算：获得一名其他角色的一张牌（手牌优先、其次装备，
/// 各区首张确定性取牌；获得进来源手牌——与拆桥弃置不同）。
#[derive(Clone)]
pub struct SnatchMachine {
    pub(crate) card: crate::ids::CardId,
    pub(crate) source: PlayerId,
    pub(crate) target: PlayerId,
    pub(crate) asked: bool,
}

impl crate::catalog::SkillMachine for SnatchMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Snatch {
            card: self.card,
            source: self.source,
            target: self.target,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            let t = ctx.core.player(self.target);
            let pick = t
                .hand
                .first()
                .copied()
                .or_else(|| t.equips.first().copied());
            if let Some(c) = pick {
                {
                    let was_equip = ctx.catalog.card_kind(c).is_equip();
                    let p = ctx.core.player_mut(self.target);
                    p.hand.retain(|x| *x != c);
                    p.equips.retain(|x| *x != c);
                    if was_equip {
                        revoke_equip_skills(ctx.core, ctx.catalog, self.target, c);
                    }
                }
                ctx.core.player_mut(self.source).hand.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: self.target,
                    to: self.source,
                });
                ctx.log(LogEvent::CardsGiven {
                    from: self.target,
                    to: self.source,
                    cards: vec![c],
                });
            }
            let card = self.card;
            ctx.to_discard(card);
            ctx.core.table.retain(|x| *x != card);
        }
        Flow::Done
    }
}

/// 铁索连环结算：无懈窗口未抵消后，逐存活目标翻转横置
/// （每目标一个 Linked 子帧，走 commit 翻转并写回载荷）。
/// 牌在出牌提交时已入手牌→桌面，本机器结算后一次性弃置——
/// 旧实现只推 Linked 帧而牌从未离手，本批修复（ADR 0705）。
#[derive(Clone)]
pub struct ChainMachine {
    card: crate::ids::CardId,
    source: PlayerId,
    targets: Vec<PlayerId>,
    /// 已发出 Linked 帧的目标数。
    emitted: usize,
}

impl ChainMachine {
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        targets: Vec<PlayerId>,
        emitted: usize,
    ) -> Self {
        Self { card, source, targets, emitted }
    }
}

impl crate::catalog::SkillMachine for ChainMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Chain {
            card: self.card,
            source: self.source,
            targets: self.targets.clone(),
            emitted: self.emitted,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        while self.emitted < self.targets.len() {
            let t = self.targets[self.emitted];
            self.emitted += 1;
            if !ctx.core.player(t).alive {
                continue; // 窗口期间死亡的目标跳过
            }
            ctx.emit(EventData::Linked {
                player: t,
                linked: false,
            });
            return Flow::Yield;
        }
        // 收尾：铁索离桌入弃牌堆（唯一弃置点）
        let card = self.card;
        ctx.to_discard(card);
        ctx.core.table.retain(|c| *c != card);
        Flow::Done
    }
}

/// 无懈可击响应窗口：包裹锦囊效果机器。
/// 自来源下家起按座位顺序询问持无懈者；出无懈即反转「抵消」状态并自其下家
/// 重新询问；窗口关闭时未抵消则委托效果机器，已抵消则锦囊入弃牌堆。
/// 窗口适用范围/整牌级窗口/跳过语义见覆盖清单行为差异表。
pub struct NullifyWindowMachine {
    card: crate::ids::CardId,
    source: PlayerId,
    nullified: bool,
    queue: Vec<PlayerId>,
    asked: bool,
    /// false = 窗口已关闭，进入效果委托阶段。
    window_open: bool,
    /// 抵消关闭时是否由本窗口弃置锦囊：单次结算锦囊（决斗/铁索/延时锦囊）
    /// 为 true；AOE 逐目标窗口为 false——由外层机器在全部目标结算后
    /// 统一弃置（N01：否则「抵消一个目标」路径同一牌进弃牌堆两次）。
    owns_disposal: bool,
    effect: Option<Box<dyn crate::catalog::SkillMachine>>,
}

impl NullifyWindowMachine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        nullified: bool,
        queue: Vec<PlayerId>,
        asked: bool,
        window_open: bool,
        owns_disposal: bool,
        effect: Option<Box<dyn crate::catalog::SkillMachine>>,
    ) -> Self {
        Self { card, source, nullified, queue, asked, window_open, owns_disposal, effect }
    }
}

impl crate::catalog::SkillMachine for NullifyWindowMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(Self {
            card: self.card,
            source: self.source,
            nullified: self.nullified,
            queue: self.queue.clone(),
            asked: self.asked,
            window_open: self.window_open,
            owns_disposal: self.owns_disposal,
            effect: self.effect.as_ref().map(|m| m.fork()),
        })
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::NullifyWindow {
            card: self.card,
            source: self.source,
            nullified: self.nullified,
            queue: self.queue.clone(),
            asked: self.asked,
            window_open: self.window_open,
            owns_disposal: self.owns_disposal,
            effect: self.effect.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 窗口阶段
        if self.window_open {
            // 询问队列清理：跳过死亡/无无懈持有者
            while let Some(p) = self.queue.first().copied() {
                let has_nullify = ctx.core.player(p).alive
                    && ctx
                        .core
                        .player(p)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Nullify);
                if !has_nullify {
                    self.queue.remove(0);
                } else {
                    break;
                }
            }
            let Some(responder) = self.queue.first().copied() else {
                // 队列耗尽：窗口关闭。弃牌责任唯一化（ADR 0705 修复）：抵消时
                // 由窗口弃置锦囊；未抵消时效果机器（决斗 finish / AOE 收尾 /
                // 铁索机器）结算结束弃置——旧实现在此无条件弃牌，叠加效果
                // 机器的弃置使非抵消路径同一牌进弃牌堆两次。
                if self.nullified {
                    // 弃置责任唯一化（ADR 0705/0706）：owns_disposal 的窗口
                    // 抵消时弃置；AOE 逐目标窗口不弃置（外层收尾统一弃置）。
                    if self.owns_disposal {
                        let card = self.card;
                        ctx.to_discard(card);
                        ctx.core.table.retain(|c| *c != card);
                    }
                    ctx.log(LogEvent::CardNullified {
                        card: self.card,
                        player: self.source,
                    });
                    return Flow::Done;
                }
                self.window_open = false;
                return self.step(ctx); // 未抵消：委托效果机器
            };
            if self.asked {
                self.asked = false;
                self.queue.remove(0);
                let answer = ctx.answer().clone();
                if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                    // 出无懈：弃牌、反转抵消状态、自其下家重新询问
                    ctx.discard_from_hand(responder, c);
                    ctx.emit(EventData::CardResponded {
                        card: c,
                        player: responder,
                    });
                    self.nullified = !self.nullified;
                    self.queue = ctx.core.seat_order_from(ctx.core.next_alive(responder));
                }
                return self.step(ctx);
            }
            self.asked = true;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                ctx.core.player(responder)
                    .hand
                    .iter()
                    .filter(|c| ctx.catalog.card_kind(**c) == CardKind::Nullify)
                    .copied()
                    .map(ChoiceOption::Card),
            );
            return Flow::WaitChoice {
                observer: responder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "nullify_window",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
        // 窗口已关闭且未抵消：全权委托效果机器
        if let Some(inner) = self.effect.as_mut() {
            let flow = inner.step(ctx);
            if matches!(flow, Flow::Done) {
                self.effect = None;
                return Flow::Done;
            }
            return flow;
        }
        Flow::Done
    }

}

/// 为锦囊构造无懈窗口并预载询问队列（无懈持有者过滤在运行时进行）。
fn wrap_nullify_window(
    ctx: &Ctx<'_>,
    card: crate::ids::CardId,
    source: PlayerId,
    effect: Box<dyn crate::catalog::SkillMachine>,
) -> NullifyWindowMachine {
    // 询问队列自来源起按座位顺序（含来源本人）——对齐旧线 alive_from_seat(actor)。
    NullifyWindowMachine {
        card,
        source,
        nullified: false,
        queue: ctx.core.seat_order_from(source),
        asked: false,
        window_open: true,
        owns_disposal: true,
        effect: Some(effect),
    }
}

/// 延时锦囊置入机器（使用时无懈窗口未抵消后启动）：
/// 将桌面上的锦囊置入目标判定区；目标在窗口期间死亡则弃置
/// （同名牌限制已在命令期校验，窗口期无改判类技能，不复查）。
#[derive(Clone)]
pub struct PlaceDelayedMachine {
    card: crate::ids::CardId,
    target: PlayerId,
    placed: bool,
}

impl PlaceDelayedMachine {
    pub(crate) fn new(card: crate::ids::CardId, target: PlayerId, placed: bool) -> Self {
        Self { card, target, placed }
    }
}

impl crate::catalog::SkillMachine for PlaceDelayedMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::PlaceDelayed {
            card: self.card,
            target: self.target,
            placed: self.placed,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.placed {
            return Flow::Done;
        }
        self.placed = true;
        let card = self.card;
        ctx.core.table.retain(|c| *c != card);
        if !ctx.core.player(self.target).alive {
            ctx.to_discard(card);
            return Flow::Done;
        }
        ctx.core.player_mut(self.target).judge.push(card);
        Flow::Done
    }
}

/// 权限型能力（无触发时机，如「鬼道」「天妒」）的空机器：
/// 永不进入触发队列与挂起槽，启动即直接结束。
#[derive(Clone)]
pub struct NoopMachine;

impl NoopMachine {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl crate::catalog::SkillMachine for NoopMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(NoopMachine)
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Noop
    }
    fn step(&mut self, _ctx: &mut Ctx<'_>) -> Flow {
        Flow::Done
    }
}

/// 判定阶段机器：对判定区延时锦囊逐张判定。
/// 排序对齐旧线 `pending_delayed_cards`（ADR 0711）：非闪电延时锦囊按
/// 放置序在前、闪电在后，**队首先消费**（先放入先判）。
///
/// 单次判定的流程（N08 首组官方技能，ADR 0712）：
/// 1. 翻判定牌；
/// 2. **改判询问**（张角「鬼道」）：持有者自判定阶段拥有者起按座位顺序
///    （含本人）逐人询问一次，可打出一张**黑色手牌**代替当前判定牌
///    （旧线含装备区牌，原生本批仅手牌，差异登记）；被替换的判定牌弃置；
/// 3. 判定牌生效（乐/兵/闪电）；
/// 4. **天妒询问**（郭嘉「天妒」）：持有者按座位顺序逐人询问，
///    可将最终判定牌收入手牌（一人获得后不再询问）。
///
/// 拥有者判定中途死亡：队列与当前判定牌逐张弃置（防御路径）。
pub struct JudgePhaseMachine {
    owner: PlayerId,
    /// 待判锦囊队列（已排序：非闪电放置序在前、闪电在后；队首先消费）。
    queue: Vec<crate::ids::CardId>,
    /// 当前判定的进行态（翻牌后到生效/获得前）。
    current: Option<JudgementProgress>,
}

/// 单次判定的进行态。
#[derive(Clone)]
pub struct JudgementProgress {
    trick: crate::ids::CardId,
    judge_card: crate::ids::CardId,
    /// 鬼道改判询问队列（座位序，自拥有者起含本人）。
    rewriters: Vec<PlayerId>,
    /// 当前改判者是否已被询问（等待应答）。
    asked: bool,
    /// 天妒持有人队列（座位序）。
    tian_du: Vec<PlayerId>,
    /// 判定牌是否已被天妒获得。
    gained: bool,
    /// 是否处于改判询问阶段（false = 天妒阶段）。
    in_rewrite: bool,
    /// Finalized 是否已记（恰好一次，效果生效前，N08-d）。
    finalized: bool,
    /// 判定牌是否仍由机器持有（未弃置、未进入手牌）。
    judge_held: bool,
}

impl Clone for JudgePhaseMachine {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner,
            queue: self.queue.clone(),
            current: self.current.clone(),
        }
    }
}

impl JudgePhaseMachine {
    pub(crate) fn new(
        owner: PlayerId,
        queue: Vec<crate::ids::CardId>,
        current: Option<JudgementProgress>,
    ) -> Self {
        Self { owner, queue, current }
    }
}

impl JudgementProgress {
    pub(crate) fn from_snap(s: &crate::snap::JudgementProgressSnap) -> Self {
        Self {
            trick: s.trick,
            judge_card: s.judge_card,
            rewriters: s.rewriters.clone(),
            asked: s.asked,
            tian_du: s.tian_du.clone(),
            gained: s.gained,
            in_rewrite: s.in_rewrite,
            finalized: s.finalized,
            judge_held: s.judge_held,
        }
    }
}

impl crate::catalog::SkillMachine for JudgePhaseMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::JudgePhase {
            owner: self.owner,
            queue: self.queue.clone(),
            current: self.current.as_ref().map(|c| {
                Box::new(crate::snap::JudgementProgressSnap {
                    trick: c.trick,
                    judge_card: c.judge_card,
                    rewriters: c.rewriters.clone(),
                    asked: c.asked,
                    tian_du: c.tian_du.clone(),
                    gained: c.gained,
                    in_rewrite: c.in_rewrite,
                    finalized: c.finalized,
                    judge_held: c.judge_held,
                })
            }),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            // 拥有者死亡（防御路径：对齐排序下能致死的闪电在队尾）：
            // 待判队列与进行中的判定牌逐张弃置。
            if !ctx.core.player(self.owner).alive {
                let mut leftover = std::mem::take(&mut self.queue);
                if let Some(cur) = self.current.take() {
                    // 仅机器仍持有的判定牌需要随死亡清理弃置；
                    // 已被天妒获得（手牌）或已随效果弃置的不重复入弃牌堆。
                    if cur.judge_held {
                        leftover.push(cur.judge_card);
                    }
                }
                if !leftover.is_empty() {
                    ctx.core.discard.extend(leftover.iter().copied());
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: std::mem::take(&mut leftover),
                    });
                }
                return Flow::Done;
            }

            // 当前判定的进行态
            if let Some(cur) = &mut self.current {
                if cur.in_rewrite {
                    // 应答消费：当前改判者出黑色手牌替换或放弃
                    if cur.asked {
                        cur.asked = false;
                        // 运行时防御：改判队列已空（正常不会出现——asked 与
                        // 非空队列同置）则跳过改判直接进入后续阶段（N04-d）。
                        if cur.rewriters.is_empty() {
                            cur.in_rewrite = false;
                            continue;
                        }
                        let rewriter = cur.rewriters.remove(0);
                        let answer = ctx.answer().clone();
                        if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                            // 打出的牌离开手牌或装备区、由判定流程持有（成为新判定牌）；
                            // 被替换的旧判定牌由改判者**获得**（对齐上游，ADR 0714）。
                            ctx.take_owned_card(rewriter, c);
                            let old = cur.judge_card;
                            ctx.core.player_mut(rewriter).hand.push(old);
                            cur.judge_card = c;
                            ctx.log(LogEvent::JudgementRewritten {
                                player: rewriter,
                                card: c,
                                replaced: old,
                            });
                        }
                        if cur.rewriters.is_empty() {
                            cur.in_rewrite = false;
                        }
                        continue;
                    }
                    // 询问下一个可用改判者（跳过死亡/无黑色牌者——手牌与
                    // 装备区均可，N08-c：仅装备区黑牌者也保持询问资格）
                    while let Some(r) = cur.rewriters.first().copied() {
                        if !ctx.core.player(r).alive {
                            cur.rewriters.remove(0);
                            continue;
                        }
                        let is_black_card = |c: crate::ids::CardId| {
                            matches!(
                                ctx.catalog.cards[c as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                        };
                        let p = ctx.core.player(r);
                        // 鬼才：任意手牌可改判（不依赖黑色）
                        let gui_cai = ctx.catalog.skill_by_name(crate::content::standard::GUI_CAI);
                        let has_gui_cai = gui_cai
                            .map(|d| ctx.core.has_skill(r, d))
                            .unwrap_or(false);
                        let has_any_hand = has_gui_cai && !p.hand.is_empty();
                        // 天道：任意手牌或装备牌可改判（hes 全量，规则 v263）
                        let tian_dao =
                            ctx.catalog.skill_by_name(crate::content::standard::BOSS_TIANDAO);
                        let has_tian_dao = tian_dao
                            .map(|d| ctx.core.has_skill(r, d))
                            .unwrap_or(false);
                        let has_any_hes =
                            has_tian_dao && (!p.hand.is_empty() || !p.equips.is_empty());
                        let has_black = p
                            .hand
                            .iter()
                            .chain(p.equips.iter())
                            .copied()
                            .any(is_black_card);
                        if !has_black && !has_any_hand && !has_any_hes {
                            cur.rewriters.remove(0);
                            continue;
                        }
                        break;
                    }
                    if cur.rewriters.is_empty() {
                        cur.in_rewrite = false;
                        continue;
                    }
                    let rewriter = cur.rewriters[0];
                    cur.asked = true;
                    // 黑色手牌 + 黑色装备牌均可改判（对齐旧线
                    // `judgment_black_rewrite_cards`：hand + equip）。
                    let mut options = vec![ChoiceOption::Pass];
                    let is_black = |id: crate::ids::CardId| {
                        matches!(
                            ctx.catalog.cards[id as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    };
                    // 鬼才持有者：全部手牌可打（不限花色）
                    let gui_cai = ctx.catalog.skill_by_name(crate::content::standard::GUI_CAI);
                    let gui_cai_holders_any = gui_cai
                        .map(|d| ctx.core.has_skill(rewriter, d))
                        .unwrap_or(false);
                    // 天道持有者：全部手牌与装备牌可打（hes 全量，规则 v263）
                    let tian_dao =
                        ctx.catalog.skill_by_name(crate::content::standard::BOSS_TIANDAO);
                    let tian_dao_holders_any = tian_dao
                        .map(|d| ctx.core.has_skill(rewriter, d))
                        .unwrap_or(false);
                    for c in ctx.core.player(rewriter).hand.iter().copied() {
                        if tian_dao_holders_any || gui_cai_holders_any || is_black(c) {
                            options.push(ChoiceOption::Card(c));
                        }
                    }
                    // 装备区：黑色牌（鬼道路径）或天道持有者任意装备
                    for c in ctx.core.player(rewriter).equips.iter().copied() {
                        if tian_dao_holders_any || is_black(c) {
                            options.push(ChoiceOption::Card(c));
                        }
                    }
                    return Flow::WaitChoice {
                        observer: rewriter,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::RespondCard,
                            prompt: "gui_dao_rewrite",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    };
                }
                // 判定牌确定（N08-d 顺序：Rewritten×N → Finalized → 效果 →
                // 天妒 Gained）：Finalized 恰好记一次、在效果与天妒询问前。
                if !cur.finalized {
                    cur.finalized = true;
                    let trick = cur.trick;
                    let judge_card = cur.judge_card;
                    // 天妒待询问时判定牌由机器持有，不在此弃置；
                    // 否则随效果弃置（judge_held 置否）。
                    if cur.gained || cur.tian_du.is_empty() {
                        cur.judge_held = false;
                    }
                    let machine_holds = cur.judge_held;
                    ctx.log(LogEvent::JudgementFinalized {
                        player: self.owner,
                        card: judge_card,
                    });
                    Self::apply_judgement(self.owner, ctx, trick, judge_card, machine_holds);
                    continue;
                }
                // 天妒阶段（效果生效后）：跳过死亡持有者；有人获得后不再询问
                while let Some(h) = cur.tian_du.first().copied() {
                    if !ctx.core.player(h).alive || cur.gained {
                        cur.tian_du.remove(0);
                    } else {
                        break;
                    }
                }
                // 天妒应答消费
                if !cur.tian_du.is_empty() && cur.asked && !cur.gained && cur.judge_held {
                    cur.asked = false;
                    cur.judge_held = false;
                    let holder = cur.tian_du.remove(0);
                    if let ChoiceOption::Confirm = ctx.answer().first().copied().unwrap_or(ChoiceOption::Pass) {
                        let card = cur.judge_card;
                        ctx.core.player_mut(holder).hand.push(card);
                        cur.gained = true;
                        ctx.log(LogEvent::JudgementGained { player: holder, card });
                    } else {
                        // 放弃获得：机器持有的判定牌弃置
                        ctx.to_discard(cur.judge_card);
                    }
                    self.current = None;
                    continue;
                }
                if cur.tian_du.is_empty() || cur.gained || !cur.judge_held {
                    // 完结：机器仍持有的判定牌（天妒者死亡等防御路径）弃置
                    if cur.judge_held && !cur.gained {
                        cur.judge_held = false;
                        ctx.to_discard(cur.judge_card);
                    }
                    self.current = None;
                    continue;
                }
                // 询问天妒持有者
                let holder = cur.tian_du[0];
                cur.asked = true;
                return Flow::WaitChoice {
                    observer: holder,
                    spec: ChoiceSpec::confirm("tian_du_gain"),
                };
            }

            // 取下一锦囊并翻判定牌
            if self.queue.is_empty() {
                return Flow::Done;
            }
            let trick = self.queue.remove(0);
            let judge_card = ctx.draw_judge_card();
            ctx.log(LogEvent::Judged {
                player: self.owner,
                card: judge_card,
            });
            let gui_dao = ctx.catalog.skill_by_name(crate::content::standard::GUI_DAO);
            let gui_cai = ctx.catalog.skill_by_name(crate::content::standard::GUI_CAI);
            let tian_du = ctx.catalog.skill_by_name(crate::content::standard::TIAN_DU);
            // 天妒（owner_is_target，对齐旧线内容定义/标准包文本「你的判定牌」）：
            // 仅判定阶段拥有者本人持有时由本人询问。
            let tian_du_self = tian_du
                .map(|d| ctx.core.has_skill(self.owner, d))
                .unwrap_or(false);
            // 改判者：自判定阶段拥有者起按座位序（含本人），逐人一次——
            // 鬼道（黑色牌改判）或鬼才（任意手牌改判）持有者均入队（对齐旧线
            // judgment_rewriters(tx, owner)；同座位双技能只入队一次）。
            // 天道（Boss 张角，规则 v263）：hes 任意牌可改判——改判资格
            // 与鬼道/鬼才并列（座位序同队）
            let tian_dao =
                ctx.catalog.skill_by_name(crate::content::standard::BOSS_TIANDAO);
            let rewriters: Vec<PlayerId> = ctx
                .core
                .seat_order_from(self.owner)
                .into_iter()
                .filter(|p| {
                    let has = |d: Option<crate::ids::SkillDefId>| {
                        d.map(|d| ctx.core.has_skill(*p, d)).unwrap_or(false)
                    };
                    has(gui_dao) || has(gui_cai) || has(tian_dao)
                })
                .collect();
            self.current = Some(JudgementProgress {
                trick,
                judge_card,
                rewriters,
                asked: false,
                tian_du: if tian_du_self {
                    vec![self.owner]
                } else {
                    Vec::new()
                },
                gained: false,
                in_rewrite: true,
                finalized: false,
                judge_held: true,
            });
        }
    }
}

impl JudgePhaseMachine {
    /// 判定牌生效：按锦囊种类套用效果（`machine_holds` 为真时判定牌仍由
    /// 机器持有——天妒未决——不随效果弃置）。
    fn apply_judgement(
        owner: PlayerId,
        ctx: &mut Ctx<'_>,
        trick: crate::ids::CardId,
        judge_card: crate::ids::CardId,
        machine_holds: bool,
    ) {
        // 国色等 view-as 转化：声明牌种优先于目录牌种（N08 迁移）
        let trick_kind = ctx
            .core
            .declared_kinds
            .get(&trick)
            .copied()
            .unwrap_or_else(|| ctx.catalog.card_kind(trick));
        // 声明消费后移除（牌已结算，残留条目会在后续洗回场景误判）
        ctx.core.declared_kinds.remove(&trick);
        let (suit, rank) = {
            let def = &ctx.catalog.cards[judge_card as usize];
            (def.suit, def.rank)
        };
        // 神裔（Boss 青龙共有技，规则 v233）：判定阶段的延时锦囊判定对
        // 持有者恒不生效（上游 mod judge event.type=="phase"：bool
        // true→false、false→null——各臂效果条件一并短路；牌去向不变：
        // 锦囊照常弃置、闪电照常传下家）
        let shenyi_voids = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_SHENYI)
            .map(|def| ctx.core.has_skill(owner, def))
            .unwrap_or(false);
        match trick_kind {
            CardKind::Le => {
                if !shenyi_voids && !suit.is_heart() {
                    ctx.core.turn.skip_play_phase = true;
                }
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::Bing => {
                if !shenyi_voids && !suit.is_club() {
                    ctx.core.turn.skip_draw_phase = true;
                }
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::HongShui => {
                // 洪水：梅花 2–9 → 3 点雷伤（登记差异：上游未标注属性）
                ctx.to_discard(trick);
                if !shenyi_voids && suit.is_club() && (2..=9).contains(&rank) {
                    if !machine_holds {
                        ctx.to_discard(judge_card);
                    }
                    ctx.emit(EventData::Damage(damage_data_with(
                        None,
                        owner,
                        3,
                        crate::event::DamageNature::Thunder,
                        None,
                    )));
                } else if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::Guiyoujie => {
                // 归有界判定（上游 judge 黑色 -3 / effect loseHp+randomDiscard）
                if !shenyi_voids
                    && matches!(
                        suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                ) {
                    ctx.to_discard(trick);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                    // 随机弃一张牌（手牌+装备均匀抽取；逐技对照批四口径）
                    let he_len = ctx.core.player(owner).hand.len()
                        + ctx.core.player(owner).equips.len();
                    if he_len > 0 {
                        let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                        let hand_n = ctx.core.player(owner).hand.len();
                        let c = if pick < hand_n {
                            ctx.core.player(owner).hand[pick]
                        } else {
                            ctx.core.player(owner).equips[pick - hand_n]
                        };
                        let in_hand = ctx.core.player(owner).hand.contains(&c);
                        if in_hand {
                            ctx.discard_from_hand(owner, c);
                        } else {
                            ctx.core
                                .player_mut(owner)
                                .equips
                                .retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                owner,
                                c,
                            );
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: owner,
                            cards: vec![c],
                        });
                    }
                }
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            CardKind::ShanDian => {
                if !shenyi_voids && suit.is_spade() && (2..=9).contains(&rank) {
                    ctx.to_discard(trick);
                    if !machine_holds {
                        ctx.to_discard(judge_card);
                    }
                    ctx.emit(EventData::Damage(damage_data_with(
                        None,
                        owner,
                        3,
                        crate::event::DamageNature::Thunder,
                        None,
                    )));
                    return;
                }
                // 未命中：传给下一位存活玩家；其判定区已有闪电则直接弃置
                // （对齐旧线 pass_lightning）。
                let next = ctx.core.next_alive(owner);
                let next_has_shandian = ctx
                    .core
                    .player(next)
                    .judge
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::ShanDian);
                if next_has_shandian {
                    ctx.to_discard(trick);
                } else {
                    ctx.core.player_mut(next).judge.push(trick);
                }
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
            _ => {
                ctx.to_discard(trick);
                if !machine_holds {
                    ctx.to_discard(judge_card);
                }
            }
        }
    }
}

/// AOE 单目标段（南蛮入侵/万箭齐发）：询问响应牌（杀/闪），Pass 或无牌
/// 则受 1 点伤害（完整伤害帧）。无牌者不询问直接受伤（跳过语义挂账见清单）。
#[derive(Clone)]
pub struct AoeTargetMachine {
    source: PlayerId,
    target: PlayerId,
    /// 响应需求（南蛮=杀，万箭=闪）。
    need: CardKind,
    asked: bool,
    /// 造成伤害的锦囊牌（DamageData.card 透传）。
    card: crate::ids::CardId,
}

impl AoeTargetMachine {
    pub(crate) fn new(
        source: PlayerId,
        target: PlayerId,
        need: CardKind,
        asked: bool,
        card: crate::ids::CardId,
    ) -> Self {
        Self { source, target, need, asked, card }
    }
}

impl crate::catalog::SkillMachine for AoeTargetMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::AoeTarget {
            source: self.source,
            target: self.target,
            need: self.need,
            asked: self.asked,
            card: self.card,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.asked {
            self.asked = false;
            let answer = ctx.answer().clone();
            if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                // 出响应牌免伤
                ctx.discard_from_hand(self.target, c);
                ctx.emit(EventData::CardResponded {
                    card: c,
                    player: self.target,
                });
                // 雷击（Boss 青龙，规则 v234）：万箭闪响应后判定雷击
                // （南蛮响应为杀，不触发）
                if ctx.catalog.card_kind(c) == CardKind::Shan {
                    releiji_on_shan(ctx, self.target);
                }
            } else {
                // 放弃 → 1 点伤害（完整结算）
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(self.source),
                    self.target,
                    1,
                    crate::event::DamageNature::Normal,
                    Some(self.card),
                )));
            }
            return Flow::Done;
        }
        let holders: Vec<crate::ids::CardId> = ctx.core.player(self.target)
            .hand
            .iter()
            .copied()
            .filter(|c| ctx.catalog.card_kind(*c) == self.need)
            .collect();
        if holders.is_empty() {
            // 无响应牌：不询问，直接受伤（跳过语义挂账见清单）
            ctx.emit(EventData::Damage(damage_data_with(
                Some(self.source),
                self.target,
                1,
                crate::event::DamageNature::Normal,
                Some(self.card),
            )));
            return Flow::Done;
        }
        self.asked = true;
        let mut options = vec![ChoiceOption::Pass];
        options.extend(holders.into_iter().map(ChoiceOption::Card));
        Flow::WaitChoice {
            observer: self.target,
            spec: ChoiceSpec {
                kind: ChoiceKind::RespondCard,
                prompt: "aoe_response",
                options,
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }
}

/// AOE 多目标外层机器：按冻结目标顺序逐个结算；每个目标结算前先开
/// **独立的无懈窗口**（逐目标窗口，抵消只作用于该目标——旧实现为整牌级
/// 单窗口，见覆盖清单已关闭的行为差异行）。目标中途死亡由存活过滤跳过，
/// 队列继续。AOE 牌由本机器在全部目标结算后一次性弃置。
pub struct AoeResolveMachine {
    card: crate::ids::CardId,
    source: PlayerId,
    /// 响应需求（南蛮=杀，万箭=闪）。
    need: CardKind,
    targets: Vec<PlayerId>,
    idx: usize,
    /// 当前目标的无懈窗口（内含响应段机器）。
    inner: Option<Box<dyn SkillMachine>>,
}

impl Clone for AoeResolveMachine {
    fn clone(&self) -> Self {
        Self {
            card: self.card,
            source: self.source,
            need: self.need,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| m.fork()),
        }
    }
}

impl AoeResolveMachine {
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        need: CardKind,
        targets: Vec<PlayerId>,
        idx: usize,
        inner: Option<Box<dyn SkillMachine>>,
    ) -> Self {
        Self { card, source, need, targets, idx, inner }
    }

    fn spawn_window(&self, ctx: &Ctx<'_>) -> Box<dyn SkillMachine> {
        let target = self.targets[self.idx];
        // AOE 逐目标窗口不拥有弃置权（N01）：外层在全部目标结算后统一弃置。
        // 询问队列自来源起（含本人），对齐旧线 alive_from_seat(actor)。
        Box::new(NullifyWindowMachine::new(
            self.card,
            self.source,
            false,
            ctx.core.seat_order_from(self.source),
            false,
            true,
            false,
            Some(Box::new(AoeTargetMachine::new(
                self.source, target, self.need, false, self.card,
            ))),
        ))
    }
}

impl crate::catalog::SkillMachine for AoeResolveMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            card: self.card,
            source: self.source,
            need: self.need,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| m.fork()),
        })
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Aoe {
            card: self.card,
            source: self.source,
            need: self.need,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            // 跳过已死亡目标（窗口未开启时）
            while self.inner.is_none()
                && self.idx < self.targets.len()
                && !ctx.core.player(self.targets[self.idx]).alive
            {
                self.idx += 1;
            }
            if self.idx >= self.targets.len() {
                // 收尾：AOE 牌离桌入弃牌堆（唯一弃置点）
                let card = self.card;
                ctx.to_discard(card);
                ctx.core.table.retain(|c| *c != card);
                return Flow::Done;
            }
            if self.inner.is_none() {
                self.inner = Some(self.spawn_window(ctx));
            }
            let flow = self.inner.as_mut().expect("checked").step(ctx);
            if matches!(flow, Flow::Done) {
                self.inner = None;
                self.idx += 1;
                continue;
            }
            return flow;
        }
    }
}

#[derive(Clone)]
pub struct DiscardToLimitMachine {
    player: PlayerId,
    need: usize,
    asked: bool,
}

impl DiscardToLimitMachine {
    pub(crate) fn new(player: PlayerId, need: usize, asked: bool) -> Self {
        Self { player, need, asked }
    }
}

impl crate::catalog::SkillMachine for DiscardToLimitMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::DiscardToLimit {
            player: self.player,
            need: self.need,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            // 沙漠绿洲（mtg 第六批，规则 v105）：基本牌不计上限——弃牌
            // 选项排除基本牌（不计入即不会被要求弃）
            let shamolvzhou =
                ctx.core.current_land == Some(crate::catalog::CardKind::Shamolvzhou);
            // 红莲（Boss 毕方，规则 v228）：弃牌选项排除红色牌（不计入
            // 上限即不会被要求弃）
            let honglianx = ctx
                .catalog
                .skill_by_name(crate::content::standard::BOSS_HONGLIANX)
                .map(|def| ctx.core.has_skill(self.player, def))
                .unwrap_or(false);
            let options: Vec<ChoiceOption> = ctx.core.player(self.player)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    if honglianx
                        && matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    {
                        return false;
                    }
                    !shamolvzhou
                        || crate::content::host::yangpijuan_class(
                            ctx.catalog.card_kind(*c),
                        ) != Some(0)
                })
                .map(ChoiceOption::Card)
                .collect();
            return Flow::WaitChoice {
                observer: self.player,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "discard_to_limit",
                    options,
                    min: self.need,
                    max: self.need,
                    allow_cancel: false,
                },
            };
        }
        let answer = ctx.answer().clone();
        let mut discarded: Vec<crate::ids::CardId> = Vec::new();
        for opt in answer {
            if let ChoiceOption::Card(c) = opt {
                ctx.discard_from_hand(self.player, c);
                discarded.push(c);
            }
        }
        if !discarded.is_empty() {
            // 水没墓穴计数（mtg 第三批，规则 v102）：弃牌阶段内弃牌
            ctx.core.turn.discard_phase_discarded += discarded.len() as u32;
            // 弃牌事件：连营类「失去手牌」触发依赖此事件（ADR 0763）
            ctx.emit(EventData::CardsDiscarded {
                player: self.player,
                cards: discarded,
            });
        }
        Flow::Done
    }
}

/// 死亡结算：清空牌区（手牌/装备/判定区）、移除技能实例、
/// 身份局奖惩与胜负判定。
///
/// 奖惩（身份局，ADR 0705）：杀死反贼摸三张；主公杀死忠臣弃掉全部手牌与
/// 装备。奖惩先于胜负判定（先结算再判终局，口径挂账见清单）。
/// 胜负（身份局正式规则）：主公死亡 → 内奸是唯一存活则内奸胜，否则反贼胜；
/// 反贼与内奸全灭 → 主公方胜。原型模式维持存活数 ≤ 1。
#[derive(Clone)]
pub struct DeathMachine {
    player: PlayerId,
    killer: Option<PlayerId>,
}

impl DeathMachine {
    pub(crate) fn new(player: PlayerId, killer: Option<PlayerId>) -> Self {
        Self { player, killer }
    }
}

impl crate::catalog::SkillMachine for DeathMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Death {
            player: self.player,
            killer: self.killer,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 死亡拦截（规则 v171 死亡取消型基建）：标记置位则跳过死亡执
        // 行（存活态保持——前置窗口技已重置体力；上游 dieBefore
        // trigger.cancel 同口径）。
        if ctx.core.player(self.player).death_cancelled {
            ctx.core.player_mut(self.player).death_cancelled = false;
            return Flow::Done;
        }
        // versus 替补（规则 v206）：死亡座位所在队替补池非空 → 弹队首
        // 换将复活（上游 revive+uninit/init+摸 4+濒死链取消同口径：死亡
        // 执行被替补替代——本座位不进入死亡结算，奖惩/终局判定不发生；
        // 登记差异：上游 dieAfter 展示与「出场」日志不入原生日志，替补
        // 出场为新将满血开局）。
        {
            let team = ctx.core.versus_team_of(self.player);
            if let Some(ti) = team {
                let has_sub = ctx
                    .core
                    .versus_substitutes()
                    .and_then(|pools| pools.get(ti))
                    .is_some_and(|pool| !pool.is_empty());
                if has_sub {
                    let g = match &mut ctx.core.mode {
                        crate::state::GameMode::Versus { substitutes, .. } => {
                            substitutes[ti].remove(0)
                        }
                        _ => unreachable!("versus_team_of 只在 Versus 模式"),
                    };
                    // 换将前清本座位技能与区域（上游 uninit/init 重建面：
                    // 手牌/装备/判定区离场入弃堆，替补从零开始）
                    let removed: Vec<crate::ids::SkillDefId> = ctx
                        .core
                        .skills
                        .keys()
                        .filter(|(o, _)| *o == self.player)
                        .map(|(_, d)| *d)
                        .collect();
                    for def in removed {
                        ctx.core.skills.remove(&(self.player, def));
                    }
                    let leftover = {
                        let p = ctx.core.player_mut(self.player);
                        let mut leftover = std::mem::take(&mut p.hand);
                        leftover.extend(std::mem::take(&mut p.equips));
                        leftover.extend(std::mem::take(&mut p.judge));
                        p.alive = true;
                        p.dying = false;
                        p.death_cancelled = false;
                        p.general_slots = [Some(g), None];
                        leftover
                    };
                    for c in leftover {
                        ctx.to_discard(c);
                    }
                    crate::engine::apply_general_assembly(
                        ctx.core,
                        ctx.catalog,
                        self.player,
                        0,
                    );
                    let p = ctx.core.player_mut(self.player);
                    p.hp = p.max_hp as i32;
                    ctx.draw_cards(self.player, 4);
                    return Flow::Done;
                }
            }
        }
        // 回火（Boss 焰灵，规则 v241）：死亡反噬标记——技能实例随死亡
        // 结算移除，故在移除前捕获
        let had_huihuo = ctx
            .catalog
            .skill_by_name(crate::content::standard::BOSS_HUIHUO)
            .map(|d| ctx.core.skills.contains_key(&(self.player, d)))
            .unwrap_or(false);
        let p = ctx.core.player_mut(self.player);
        p.alive = false;
        p.dying = false;
        p.hp = 0;
        let mut leftover = std::mem::take(&mut p.hand);
        leftover.extend(std::mem::take(&mut p.equips));
        leftover.extend(std::mem::take(&mut p.judge));
        for c in leftover {
            ctx.to_discard(c);
        }
        let removed: Vec<crate::ids::SkillDefId> = ctx
            .core
            .skills
            .keys()
            .filter(|(o, _)| *o == self.player)
            .map(|(_, d)| *d)
            .collect();
        for def in removed {
            ctx.core.skills.remove(&(self.player, def));
            ctx.log(LogEvent::SkillToggled {
                player: self.player,
                skill: ctx.catalog.skill_name(def),
                enabled: false,
            });
        }
        ctx.log(LogEvent::PlayerDied {
            player: self.player,
            killer: self.killer,
        });
        // 造异（Boss 颛顼，规则 v246）：玄冥/共工死亡且场上无存活同名
        // 将 → 造异持有者摸 4 并获「造异异变」（运行时授予；技能实例已
        // 移除，故以死亡者武将名判定）
        {
            let dead_name = (|| {
                let g = ctx.core.player(self.player).general_slots[0];
                let g = g?;
                let base = (ctx.catalog.general_count()
                    - crate::content::standard::BOSS_GENERALS.len())
                    as u16;
                if g < base {
                    return None;
                }
                ctx.catalog.general_meta(g).map(|meta| meta.0.to_string())
            })();
            if dead_name.as_deref() == Some("共工")
                || dead_name.as_deref() == Some("玄冥")
            {
                let base = (ctx.catalog.general_count()
                    - crate::content::standard::BOSS_GENERALS.len())
                    as u16;
                let named_alive = (base..ctx.catalog.general_count() as u16)
                    .filter(|&d| {
                        matches!(
                            ctx.catalog.general_meta(d).map(|m| m.0),
                            Some(n) if n == "共工" || n == "玄冥"
                        )
                    })
                    .any(|gd| {
                        (0..ctx.core.players.len() as u8).any(|t| {
                            ctx.core.player(t).alive
                                && ctx.core.player(t).general_slots[0] == Some(gd)
                        })
                    });
                if !named_alive {
                    let zdef = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_ZAOYI);
                    if let Some(zdef) = zdef {
                        let holders: Vec<PlayerId> = ctx
                            .core
                            .skills
                            .iter()
                            .filter(|((_, dd), i)| *dd == zdef && i.enabled)
                            .map(|((o, _), _)| *o)
                            .filter(|o| ctx.core.player(*o).alive)
                            .collect();
                        for holder in holders {
                            ctx.draw_cards(holder, 4);
                            ctx.core.skills.insert(
                                (holder, zhp_def(ctx)),
                                crate::state::SkillInstance {
                                    owner: holder,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                            ctx.log(LogEvent::SkillToggled {
                                player: holder,
                                skill: ctx.catalog.skill_name(zhp_def(ctx)),
                                enabled: true,
                            });
                        }
                    }
                }
            }
        }
        // 回火（Boss 焰灵，规则 v241）：死亡结算完成后其敌方全体各受
        // 3 点无来源火伤（上游 boss_huihuo2 global dieAfter 同口径）
        if had_huihuo {
            let seats = ctx.core.players.len() as PlayerId;
            for t in 0..seats {
                if !ctx.core.player(t).alive {
                    continue;
                }
                if ctx.core.is_boss_side(t) != ctx.core.is_boss_side(self.player) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        None,
                        t,
                        3,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                }
            }
        }
        // 身份局奖惩：先于胜负判定
        if let Some(roles) = ctx.core.identity_roles() {
            let dead_role = roles[self.player as usize];
            if let Some(k) = self.killer {
                if k != self.player && ctx.core.player(k).alive {
                    match dead_role {
                        crate::state::Role::Rebel => {
                            ctx.draw_cards(k, 3);
                        }
                        crate::state::Role::Loyalist
                            if roles[k as usize] == crate::state::Role::Lord =>
                        {
                            // 主公杀忠臣：弃掉全部手牌与装备
                            let lord = ctx.core.player_mut(k);
                            let mut cards = std::mem::take(&mut lord.hand);
                            let lord_equips = std::mem::take(&mut lord.equips);
                            for c in &lord_equips {
                                revoke_equip_skills(ctx.core, ctx.catalog, k, *c);
                            }
                            cards.extend(lord_equips);
                            ctx.core.discard.extend(cards.clone());
                            if !cards.is_empty() {
                                ctx.log(LogEvent::CardDiscarded { player: k, cards });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // 对决 2v2 奖惩（P2/N08 模式入口批一，规则 v80）：先于胜负判定——
        // 杀死敌方摸三张；杀死友方弃掉自己全部手牌与装备（上游 versus
        // 2v2 基线；替补机制未建模）。
        if let Some(teams) = ctx.core.versus_teams().map(|t| t.to_vec()) {
            if let Some(k) = self.killer {
                if k != self.player && ctx.core.player(k).alive {
                    let dead_team = teams.iter().position(|t| t.contains(&self.player));
                    let killer_team = ctx.core.versus_team_of(k);
                    if let (Some(dt), Some(kt)) = (dead_team, killer_team) {
                        if dt != kt {
                            ctx.draw_cards(k, 3);
                        } else {
                            // 杀友方：弃掉自己全部手牌与装备
                            let killer = ctx.core.player_mut(k);
                            let mut cards = std::mem::take(&mut killer.hand);
                            let killer_equips = std::mem::take(&mut killer.equips);
                            for c in &killer_equips {
                                revoke_equip_skills(ctx.core, ctx.catalog, k, *c);
                            }
                            cards.extend(killer_equips);
                            ctx.core.discard.extend(cards.clone());
                            if !cards.is_empty() {
                                ctx.log(LogEvent::CardDiscarded { player: k, cards });
                            }
                        }
                    }
                }
            }
        }
        // 胜负判定
        match ctx.core.mode.clone() {
            crate::state::GameMode::Prototype => {
                let last = ctx.core.alive_players().next();
                if ctx.core.alive_count() <= 1 {
                    if let Some(w) = last {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Player(w),
                        });
                    }
                }
            }
            crate::state::GameMode::Identity { roles } => {
                let lord = roles
                    .iter()
                    .position(|r| *r == crate::state::Role::Lord)
                    .expect("identity mode has a lord");
                let lord_alive = ctx.core.player(lord as PlayerId).alive;
                if !lord_alive {
                    // 主公死亡：内奸是唯一存活 → 内奸胜；否则反贼胜
                    let traitor_sole = ctx.core.alive_count() == 1
                        && roles.iter().enumerate().any(|(i, r)| {
                            *r == crate::state::Role::Traitor
                                && ctx.core.player(i as PlayerId).alive
                        });
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: if traitor_sole {
                            crate::state::Winner::Traitor
                        } else {
                            crate::state::Winner::Rebels
                        },
                    });
                } else {
                    // 反贼与内奸全灭 → 主公方胜
                    let rebels_gone = !roles.iter().enumerate().any(|(i, r)| {
                        matches!(
                            r,
                            crate::state::Role::Rebel | crate::state::Role::Traitor
                        ) && ctx.core.player(i as PlayerId).alive
                    });
                    if rebels_gone {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::LordSide,
                        });
                    }
                }
            }
            crate::state::GameMode::Kingdoms => {
                // 国战（N11 M3 门控，ADR 0756/0757）：存活角色全部同一
                // **已声明**势力（主将明置优先，主暗副明按副将）→ 该势力胜；
                // 任一存活角色势力未定（双暗置/槽空）→ 不判负，对局继续。
                let mut factions: Vec<crate::catalog::Faction> = Vec::new();
                for p in ctx.core.alive_players() {
                    let Some(f) = ctx.core.declared_faction(ctx.catalog, p) else {
                        return Flow::Done;
                    };
                    if !factions.contains(&f) {
                        factions.push(f);
                    }
                }
                if factions.len() == 1 {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Faction(factions[0]),
                    });
                }
            }
            crate::state::GameMode::Versus { teams, .. } => {
                // 对决（P2/N08 模式入口批一）：任一队伍全灭 → 对方队伍胜
                for (i, t) in teams.iter().enumerate() {
                    if t.iter().all(|p| !ctx.core.player(*p).alive) {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Versus(1 - i),
                        });
                        break;
                    }
                }
            }
            crate::state::GameMode::Doudizhu { lord } => {
                // 斗地主（P2/N08 模式入口批二）：地主死亡 → 农民胜；
                // 农民全灭 → 地主胜（上游 game.over(me===game.zhu) 语义；
                // 无击杀奖惩——dieAfter 仅 UI+判负）
                let lord_alive = ctx.core.player(lord).alive;
                if !lord_alive {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Doudizhu { lord_won: false },
                    });
                } else {
                    let farmers_gone = ctx
                        .core
                        .alive_players()
                        .all(|p| p == lord);
                    if farmers_gone {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Doudizhu { lord_won: true },
                        });
                    }
                }
            }
            crate::state::GameMode::Kof { mut pools, mut wins, games } => {
                // KOF 引擎层（P2/N08 模式入口批四，规则 v83）：2 人局，
                // 唯一存活者胜并推进会话——败者队首淘汰、胜者队首轮转
                //（移至队尾）、比分/局数 +1。无存活者（理论不可达）不推进。
                let last = ctx.core.alive_players().next();
                if let Some(w) = last {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Player(w),
                    });
                    let wi = w as usize;
                    let li = 1 - wi;
                    if !pools[li].is_empty() {
                        pools[li].remove(0);
                    }
                    if !pools[wi].is_empty() {
                        let kept = pools[wi].remove(0);
                        pools[wi].push(kept);
                    }
                    wins[wi] += 1;
                    // games 经可变借用直接更新
                    ctx.core.mode = crate::state::GameMode::Kof {
                        pools,
                        wins,
                        games: games + 1,
                    };
                }
            }
            crate::state::GameMode::Boss { boss, fellows, .. } => {
                // Boss（P2/N08 模式入口批三-a）：boss 死亡 → 挑战者胜；
                // 挑战者全灭 → boss 胜（上游 checkResult 同构）。
                let boss_alive = ctx.core.player(boss).alive;
                if !boss_alive {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Boss { boss_won: false },
                    });
                } else {
                    let challengers_gone = ctx
                        .core
                        .alive_players()
                        .all(|p| p == boss || fellows.contains(&p));
                    if challengers_gone {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Boss { boss_won: true },
                        });
                    }
                }
            }
        }
        Flow::Done
    }
}

/// E03 猴子偷桃：使用前取消窗口（命令层挂起机器，ActivateSkill 槽模式）。
/// 他人出桃且装备猴子时拦截提交：逐持有者确认——确认者弃猴子并把桃收为
/// 手牌（使用不发生，无 CardUsed 帧）；全部放弃则按普通桃结算（回血、
/// 入弃牌堆）。持有者死亡或失猴即跳过。
#[derive(Clone)]
pub struct MonkeyStealMachine {
    pub(crate) card: crate::ids::CardId,
    pub(crate) user: PlayerId,
    pub(crate) holders: Vec<PlayerId>,
    pub(crate) asked: bool,
}

impl MonkeyStealMachine {
    pub(crate) fn new(
        card: crate::ids::CardId,
        user: PlayerId,
        holders: Vec<PlayerId>,
    ) -> Self {
        Self { card, user, holders, asked: false }
    }
}

impl crate::catalog::SkillMachine for MonkeyStealMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::MonkeySteal {
            card: self.card,
            user: self.user,
            holders: self.holders.clone(),
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            // 跳过死亡或已失去猴子的持有者。
            while let Some(h) = self.holders.first().copied() {
                let def = ctx
                    .catalog
                    .skill_by_name(crate::content::host::HUANLE_MONKEY_KEY);
                let armed = def.is_some_and(|d| ctx.core.has_skill(h, d));
                if !ctx.core.player(h).alive || !armed {
                    self.holders.remove(0);
                } else {
                    break;
                }
            }
            let Some(holder) = self.holders.first().copied() else {
                // 全部放弃：按普通桃结算（回血、入弃牌堆）。
                ctx.to_discard(self.card);
                ctx.recover(self.user, 1, Some(self.card));
                return Flow::Done;
            };
            if self.asked {
                self.asked = false;
                self.holders.remove(0);
                let confirmed = ctx
                    .answer()
                    .first()
                    .map(|o| *o == ChoiceOption::Confirm)
                    .unwrap_or(false);
                if confirmed {
                    // 取消使用：桃从使用者手牌收为持有者手牌，弃猴子
                    //（离场撤销装备技）。
                    let user_hand = &mut ctx.core.players[self.user as usize].hand;
                    let pos = user_hand
                        .iter()
                        .position(|c| *c == self.card)
                        .expect("被拦截的桃应在使用者手牌");
                    let peach = user_hand.remove(pos);
                    ctx.core.players[holder as usize].hand.push(peach);
                    ctx.log(LogEvent::CardsGiven {
                        from: self.user,
                        to: holder,
                        cards: vec![peach],
                    });
                    if let Some(pos) = ctx.core.players[holder as usize]
                        .equips
                        .iter()
                        .position(|c| {
                            ctx.catalog.card_kind(*c) == CardKind::Monkey
                        })
                    {
                        let monkey = ctx.core.players[holder as usize].equips.remove(pos);
                        revoke_equip_skills(
                            &mut ctx.core,
                            ctx.catalog,
                            holder,
                            monkey,
                        );
                        ctx.core.discard.push(monkey);
                        ctx.log(LogEvent::CardDiscarded {
                            player: holder,
                            cards: vec![monkey],
                        });
                    }
                    return Flow::Done; // 使用被取消：无 CardUsed 帧。
                }
                continue; // 放弃 → 队列下一持有者。
            }
            self.asked = true;
            return Flow::WaitChoice {
                observer: holder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "huanle_monkey_confirm",
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
    }
}
