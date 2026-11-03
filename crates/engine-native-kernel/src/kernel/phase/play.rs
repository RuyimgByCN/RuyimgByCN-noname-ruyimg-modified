use super::super::*;

pub(super) fn begin_play(ctx: &mut Ctx<'_>, player: PlayerId) {
    // 往生（十殿阎罗转轮王，规则 v269）：出牌阶段开始——视为
    // 随机使用一张【南蛮入侵】/【万箭齐发】（上游 boss_wangsheng
    // useCard noai；randomGet→恒定取南蛮确定性近似登记；
    // use_virtual_card 载体管线，目标=全部其他存活，机器直发
    // 不经距离/目标数校验=乱剑同口径）
    if let Some(wsdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_WANGSHENG)
    {
        if ctx.core.has_skill(player, wsdef) {
            let seats = ctx.core.players.len() as PlayerId;
            let targets: Vec<PlayerId> = (0..seats)
                .filter(|&t| t != player && ctx.core.player(t).alive)
                .collect();
            if !targets.is_empty() {
                ctx.use_virtual_card(player, crate::catalog::CardKind::SavageAttack, targets);
            }
        }
    }
    // 星囚（Boss 蓐收，规则 v239）：出牌阶段开始交替横置全
    // 部敌方（上游 boss_xingqiu phaseUseBegin storage 取反+
    // content link 同口径；只链不解——已横置保持；对 boss_
    // mingxingzhu 授 boss_jiding 面依赖武将未入库，登记差异）
    if let Some(xqdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XINGQIU)
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
                    ctx.log(LogEvent::Linked {
                        player: t,
                        linked: true,
                    });
                }
            }
        }
    }
    // 兽九（Boss 相柳，规则 v222）：出牌阶段开始补至 9（上游
    // phaseUseBegin draw(9-count) 同口径）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JIUSHOU)
    {
        if ctx.core.has_skill(player, def) {
            let n = ctx.core.player(player).hand.len();
            if n < 9 {
                ctx.start_draw(player, (9 - n) as u32);
            }
        }
    }
    ctx.core.turn.sha_used_play_phase = false;
    // 极略（swd 批 136，规则 v315）：出牌阶段开始清零已用次数
    ctx.core.player_mut(player).jilve_used = 0;
    // 突击计数（U07 ow，规则 v118）：出牌阶段开始清零
    ctx.core.turn.cards_used_play_phase = 0;
    ctx.core.vm_aux.once.retain(|(_, _, scope)| *scope != 4);
    // 三类计数同步清零（T8 增量 14）
    ctx.core.turn.bte_used_play_phase = 0;
    // 猛态：出牌阶段已发生（跳过路径不清位——上游
    // phaseUseBegin 同口径）
    ctx.core.turn.mengtai_use_pending = false;
    // 魔剑（Boss 夜叉，规则 v253）：出牌阶段开始视为对全部
    // 敌方使用【万箭齐发】（虚拟牌机器直发——异侧存活全目标；
    // 上游 canUse 过滤同口径）
    if let Some(mjdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_MOJIAN)
    {
        if ctx.core.has_skill(player, mjdef) {
            let seats = ctx.core.players.len() as PlayerId;
            let targets: Vec<PlayerId> = (0..seats)
                .filter(|&t| {
                    ctx.core.player(t).alive
                        && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(player)
                })
                .collect();
            if !targets.is_empty() {
                ctx.use_virtual_card(player, CardKind::ArcheryAttack, targets);
            }
        }
    }
    // 御兽（Boss 罗刹，规则 v254）：出牌阶段开始视为对全部敌
    // 方使用【南蛮入侵】（魔剑同款虚拟南蛮）
    if let Some(ysdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_YUSHOU)
    {
        if ctx.core.has_skill(player, ysdef) {
            let seats = ctx.core.players.len() as PlayerId;
            let targets: Vec<PlayerId> = (0..seats)
                .filter(|&t| {
                    ctx.core.player(t).alive
                        && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(player)
                })
                .collect();
            if !targets.is_empty() {
                ctx.use_virtual_card(player, CardKind::SavageAttack, targets);
            }
        }
    }
}
