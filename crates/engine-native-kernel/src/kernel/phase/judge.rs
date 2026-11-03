use super::super::*;

pub(super) fn begin_judge(_ctx: &mut Ctx<'_>, _player: PlayerId) {    // R05（K25/A2）：本函数原 `!turn_opened` 门两段拆分——重置段前移
    // `turn_handoff`（kernel/events.rs `reset_new_turn_state`），准备阶
    // 段 Boss 技段迁 `begin_prepare`（上游 phaseZhunbeiBegin）。
    // 废墟地图（K23，A2）：已迁 TurnBegin 窗 Commit（kernel/events.rs
    // commit_turn_begin——上游 player phaseBegin silent；原锚判定开始会在
    // 额外判定阶段重复执行）。
}

/// R05（A2 审计 K26–K45 族）：准备阶段开始入场效果——原锚判定开始
/// （`begin_judge` 的 `!turn_opened` 门内后半段），按上游 phaseZhunbeiBegin
/// 语义迁至 Phase{Prepare, begin} 入场。
pub(super) fn begin_prepare(ctx: &mut Ctx<'_>, player: PlayerId) {        // 绝红（Boss 共工，规则 v244）：准备阶段——敌方全
        // 体逐个：有牌者，有装备 → 装备全弃（撤技能+入弃堆）；
        // 无装备 → 随机弃一张手牌（随机→手牌首张确定性近似
        // 登记先例族）
        if let Some(jhdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_JUEHONG)
        {
            if ctx.core.has_skill(player, jhdef) {
                let seats = ctx.core.players.len() as PlayerId;
                for t in 0..seats {
                    if t == player || !ctx.core.player(t).alive {
                        continue;
                    }
                    if ctx.core.is_boss_side(t) == ctx.core.is_boss_side(player) {
                        continue;
                    }
                    let has_cards = !ctx.core.player(t).hand.is_empty()
                        || !ctx.core.player(t).equips.is_empty();
                    if !has_cards {
                        continue;
                    }
                    let equips: Vec<crate::ids::CardId> =
                        ctx.core.player(t).equips.iter().copied().collect();
                    if !equips.is_empty() {
                        for c in equips {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            revoke_equip_skills(ctx.core, ctx.catalog, t, c);
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
            .skill_by_name(crate::vocabulary::names::BOSS_JIAZU)
        {
            if ctx.core.has_skill(player, jzdef) {
                for t in [ctx.core.ring_prev(player), ctx.core.ring_next(player)] {
                    if t == player || !ctx.core.player(t).alive {
                        continue;
                    }
                    if ctx.core.is_boss_side(t) == ctx.core.is_boss_side(player) {
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
                                ctx.catalog.card_slot_of(ctx.core, *c),
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
            .skill_by_name(crate::vocabulary::names::BOSS_ZAOYI_HP)
        {
            if ctx.core.has_skill(player, zhdef) {
                let seats = ctx.core.players.len() as PlayerId;
                let enemies: Vec<PlayerId> = (0..seats)
                    .filter(|&t| {
                        ctx.core.player(t).alive
                            && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(player)
                    })
                    .collect();
                if let Some(min) = enemies.iter().map(|&t| ctx.core.player(t).hp).min() {
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
        // 自润（Boss 玄冥，规则 v247）：准备阶段——全体角色
        // 按座位序各摸牌：有装备者摸 2、无装备者摸 1（上游
        // boss_zirun phaseZhunbeiBegin asyncDraw 同口径）
        if let Some(zrdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_ZIRUN)
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
                    ctx.start_draw(t, n as u32);
                }
            }
        }
        // 魔道（Boss 夜叉，规则 v253）：准备阶段摸 2 张牌
        if let Some(mddef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_MODAO)
        {
            if ctx.core.has_skill(player, mddef) {
                ctx.start_draw(player, 2);
            }
        }
        // 轮回（十殿阎罗转轮王，规则 v269）：准备阶段，若体力
        // ≤2——与体力最高且大于 2 的一名其他角色交换体力值
        //（上游 boss_lunhui hp 直写 min(maxHp) 双封顶同口径；
        // randomGet→座位序首个确定性近似登记；无事件面直写）
        if let Some(lhdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_LUNHUI)
        {
            if ctx.core.has_skill(player, lhdef) && ctx.core.player(player).hp <= 2 {
                let seats = ctx.core.players.len() as PlayerId;
                let cands: Vec<i32> = (0..seats)
                    .filter(|&t| {
                        t != player && ctx.core.player(t).alive && ctx.core.player(t).hp > 2
                    })
                    .map(|t| ctx.core.player(t).hp)
                    .collect();
                let best = cands.iter().copied().max().and_then(|maxhp| {
                    (0..seats).find(|&t| {
                        t != player && ctx.core.player(t).alive && ctx.core.player(t).hp == maxhp
                    })
                });
                if let Some(t) = best {
                    let hp1 = ctx.core.player(player).hp;
                    let hp2 = ctx.core.player(t).hp;
                    let my_max = ctx.core.player(player).max_hp as i32;
                    let t_max = ctx.core.player(t).max_hp as i32;
                    ctx.core.player_mut(player).hp = hp2.min(my_max);
                    ctx.core.player_mut(t).hp = hp1.min(t_max);
                }
            }
        }
        // 猛态（Boss 年兽，规则 v248）：准备阶段置双旗标
        //（上游 boss_mengtai_begin storage 置 true 同口径；
        // 摸牌/出牌阶段开始时清位）
        if let Some(mtdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_MENGTAI)
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
            .skill_by_name(crate::vocabulary::names::BOSS_RUIZHI)
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
                    let keep_hand = ctx.core.player(t).hand.first().copied();
                    let keep_equip = ctx.core.player(t).equips.first().copied();
                    let extra_hand: Vec<crate::ids::CardId> =
                        ctx.core.player(t).hand.iter().skip(1).copied().collect();
                    for c in extra_hand {
                        ctx.discard_from_hand(t, c);
                    }
                    let extra_equip: Vec<crate::ids::CardId> =
                        ctx.core.player(t).equips.iter().skip(1).copied().collect();
                    for c in extra_equip {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        revoke_equip_skills(ctx.core, ctx.catalog, t, c);
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
            .skill_by_name(crate::vocabulary::names::BOSS_GUIJI)
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
            .skill_by_name(crate::vocabulary::names::BOSS_GONGZHENG)
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
            .skill_by_name(crate::vocabulary::names::BOSS_XIXING)
        {
            if ctx.core.has_skill(player, xxdef) {
                let seats = ctx.core.players.len() as PlayerId;
                let linked: Vec<PlayerId> = (0..seats)
                    .filter(|&t| {
                        t != player && ctx.core.player(t).alive && ctx.core.player(t).linked
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
            .skill_by_name(crate::vocabulary::names::BOSS_LEIZHOU)
        {
            if ctx.core.has_skill(player, lzdef) {
                let seats = ctx.core.players.len() as PlayerId;
                if let Some(t) = (0..seats).find(|&t| t != player && ctx.core.player(t).alive) {
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
        // 苦酒（十殿阎罗泰山王，规则 v268）：其他角色的准备阶
        // 段——其失去 1 点体力，然后视为使用一张【酒】（上游
        // global phaseZhunbeiBegin boss_kujiu；useCard jiu 同口
        // 径置位 jiu_active 不占 jiu_used；无 useCard 事件面=登
        // 记差异）
        if let Some(kjdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_KUJIU)
        {
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == kjdef && i.enabled)
                .map(|((o, _), _)| *o)
                .collect();
            if holders
                .iter()
                .any(|h| *h != player && ctx.core.player(*h).alive)
            {
                ctx.emit(EventData::LoseHp { player, amount: 1 });
                ctx.core.turn.jiu_active = true;
            }
        }
        // 不佛（Boss 地藏王，规则 v274）：准备阶段——距离
        // 1 以内的其他存活角色受 1 点火焰伤害（有来源同上游
        // target.damage(1, player, "fire")；炼狱同款臂内循环）
        if let Some(bfdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_BUFO)
        {
            if ctx.core.has_skill(player, bfdef) {
                let seats = ctx.core.players.len() as PlayerId;
                for t in 0..seats {
                    if t == player || !ctx.core.player(t).alive {
                        continue;
                    }
                    if ctx.core.computed_distance(ctx.catalog, player, t) <= 1 {
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
        }
        // 无量（Boss 地藏王，规则 v274）：准备阶段体力小于 3
        // 时回复至 3 点（上游 boss_wuliang recover(3-hp) 同口
        // 径；ctx.recover 落 hp——v234 坑位）
        if let Some(wldef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_WULIANG)
        {
            if ctx.core.has_skill(player, wldef)
                && ctx.core.player(player).alive
                && ctx.core.player(player).hp < 3
                && ctx.core.player(player).hp < ctx.core.player(player).max_hp as i32
            {
                let amount = (3 - ctx.core.player(player).hp)
                    .min(ctx.core.player(player).max_hp as i32 - ctx.core.player(player).hp)
                    as u32;
                ctx.recover(player, amount, None);
            }
        }
        // 炙焚（Boss 都市王，规则 v273）：准备阶段——随机
        // 一名其他角色获得其一张随机手牌（若有）并受 1 点火
        // 焰伤害（上游 gainPlayerCard randomGet+damage 无来
        // 源→座位序首个+首张确定性近似登记）
        if let Some(zfdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_ZHIFEN)
        {
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == zfdef && i.enabled)
                .map(|((o, _), _)| *o)
                .collect();
            if let Some(holder) = holders.first() {
                // 上游 trigger player: phaseZhunbeiBegin——仅
                // 持有者本人准备阶段触发（他人阶段开始不触发）
                if *holder == player && ctx.core.player(*holder).alive {
                    let seats = ctx.core.players.len() as PlayerId;
                    if let Some(t) = (0..seats).find(|&t| t != *holder && ctx.core.player(t).alive)
                    {
                        // 获得其首张手牌（若有）
                        let taken = ctx.core.player(t).hand.first().copied();
                        if let Some(c) = taken {
                            ctx.core.player_mut(t).hand.retain(|&x| x != c);
                            ctx.core.player_mut(*holder).hand.push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from: t,
                                to: *holder,
                                cards: vec![c],
                            });
                        }
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            None,
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    }
                }
            }
        }
        // 锁足（Boss 平等王，规则 v272）：准备阶段——全部
        // 其他存活角色横置（上游 boss_suozu content link 同
        // 口径；不分阵营；索链甲/星囚 linked 直写同款）
        if let Some(szdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_SUOZU)
        {
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == szdef && i.enabled)
                .map(|((o, _), _)| *o)
                .collect();
            if holders
                .iter()
                .any(|h| *h == player && ctx.core.player(*h).alive)
            {
                let seats = ctx.core.players.len() as PlayerId;
                for t in 0..seats {
                    if t == player || !ctx.core.player(t).alive || ctx.core.player(t).linked {
                        continue;
                    }
                    ctx.core.player_mut(t).linked = true;
                    ctx.log(LogEvent::Linked {
                        player: t,
                        linked: true,
                    });
                }
            }
        }
        // 往事（十殿阎罗孟婆，规则 v270）：其他角色的回合开始
        // 时——其本回合不能使用或打出一种类型的牌（上游
        // boss_wangshi randomGet basic/trick/equip→恒定基本牌
        // 确定性近似登记；敌方面→全部其他近似；PlayCard 拒绝
        // 位+回合结束全清）
        if let Some(wsdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_WANGSHI)
        {
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == wsdef && i.enabled)
                .map(|((o, _), _)| *o)
                .collect();
            if holders
                .iter()
                .any(|h| *h != player && ctx.core.player(*h).alive)
            {
                ctx.core.player_mut(player).wangshi_ban = true;
            }
        }
        // 中毒（Boss 书精授技，规则 v220）：回合开始判定——
        // 非红桃受 1 点无来源伤害；非黑桃失去此技（上游
        // boss_zhongdu phaseZhunbeiBegin judge 同口径；确定性
        // 判定=铁骑内联先例）
        if let Some(zd) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_ZHONGDU)
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
                            None, player, 1,
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

