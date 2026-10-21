use super::super::*;

pub(super) fn begin_end(ctx: &mut Ctx<'_>, player: PlayerId) {
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
                if t == player || !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
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
                t != player && ctx.core.player(t).alive && ctx.core.player(t).hp >= hp_self
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
    // 天狱（群雄乱入 Boss 庞统，规则 v277）：结束阶段——自
    // 己已横置先解除，再将全部未横置的其他存活横置（上游
    // boss_tianyu link 同口径；索链甲/锁足 linked 直写同款）
    if let Some(tydef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_TIANYU)
    {
        if ctx.core.has_skill(player, tydef) {
            if ctx.core.player(player).linked {
                ctx.core.player_mut(player).linked = false;
                ctx.log(LogEvent::Linked {
                    player,
                    linked: false,
                });
            }
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
    // 拘魂（Boss 秦广王，规则 v276）：结束阶段——座位序首
    // 个其他翻面（上游 ["turnOver","link"] randomGet→恒定翻
    // 面+首个确定性近似登记；风雪刃翻面写同款）
    if let Some(jhdef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_JUHUN)
    {
        if ctx.core.has_skill(player, jhdef) {
            let seats = ctx.core.players.len() as PlayerId;
            if let Some(t) = (0..seats).find(|&t| t != player && ctx.core.player(t).alive) {
                if !ctx.core.player(t).face_down {
                    let flipped = {
                        let p = ctx.core.player_mut(t);
                        p.face_down = true;
                        p.face_down
                    };
                    ctx.emit(EventData::TurnOvered {
                        player: t,
                        face_down: flipped,
                    });
                }
            }
        }
    }
    // 无量（Boss 地藏王，规则 v274）：结束阶段摸 2 张牌
    //（上游 boss_wuliang phaseJieshuBegin draw 2 同口径）
    if let Some(wldef) = ctx
        .catalog
        .skill_by_name(crate::content::standard::BOSS_WULIANG)
    {
        if ctx.core.has_skill(player, wldef) {
            ctx.draw_cards(player, 2);
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
            if let Some(t) = (0..seats).find(|&t| t != player && ctx.core.player(t).alive) {
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 2,
                });
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
                        matches!(ctx.catalog.cards[*c as usize].kind, HorsePlus | HorseMinus)
                    })
            }) {
                let pick = ctx.core.player(t).equips.iter().copied().find(|c| {
                    matches!(ctx.catalog.cards[*c as usize].kind, HorsePlus | HorseMinus)
                });
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
            if let Some(t) = (0..seats).find(|&t| t != player && ctx.core.player(t).alive) {
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
                if ctx.core.player(t).alive && ctx.core.player(t).face_down {
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
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
        if ctx.core.has_skill(player, ssdef) && ctx.core.player(player).hp > 1 {
            let seats = ctx.core.players.len() as PlayerId;
            for t in 0..seats {
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
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
            if let Some(t) = (0..seats)
                .find(|&t| t != player && ctx.core.player(t).alive && !ctx.core.player(t).boss_mad)
            {
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
        if ctx.core.has_skill(player, mtdef) && ctx.core.turn.mengtai_draw_pending {
            ctx.core.turn.mengtai_draw_pending = false;
            ctx.draw_cards(player, 3);
        }
    }
    // 封冻解除（Boss 应昭，规则 v228）：回合结束全清封印
    for p in ctx.core.players.iter_mut() {
        p.fengyin_active = false;
    }
    // 反噬 terra 清零（十殿阎罗转轮王，规则 v269）：回合结束
    // 全清（上游 addTempSkill 当前回合结束同口径）
    for p in ctx.core.players.iter_mut() {
        p.zlfanshi_terra = false;
    }
    // 往事封禁清零（十殿阎罗孟婆，规则 v270）：回合结束全清
    //（上游 addTempSkill boss_wangshi2 当前回合结束同口径）
    for p in ctx.core.players.iter_mut() {
        p.wangshi_ban = false;
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
            if ctx.core.player(holder).alive && tanyu_is_max_hand(ctx.core, holder) {
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
            ctx.use_virtual_card(holder, CardKind::ArcheryAttack, targets);
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
