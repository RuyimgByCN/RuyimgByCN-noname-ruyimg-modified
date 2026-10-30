use super::super::*;

pub(super) fn begin_draw(ctx: &mut Ctx<'_>, player: PlayerId) {
    if ctx.core.turn.skip_draw_phase {
        // 兵粮寸断：跳过摸牌
        ctx.core.turn.skip_draw_phase = false;
        ctx.record_phase_skipped(player);
    } else if ctx.core.player(player).skip_next_draw {
        // 归烬牌（U05 xianjian 明绣，规则 v160）：跳过本次摸
        // 牌阶段（上游 skip("phaseDraw") 同口径）
        ctx.core.player_mut(player).skip_next_draw = false;
        ctx.record_phase_skipped(player);
    } else if ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JIUSHOU)
        .map(|def| ctx.core.has_skill(player, def))
        .unwrap_or(false)
    {
        // 兽九（Boss 相柳，规则 v222）：跳过摸牌阶段（上游
        // boss_jiushou phaseDraw trigger.cancel 同口径）
        ctx.record_phase_skipped(player);
    } else {
        // 猛态：摸牌阶段已发生（跳摸路径不清位——上游
        // phaseDrawBegin 未触发同口径）
        ctx.core.turn.mengtai_draw_pending = false;
        // 玉如意（swd 卡牌面补切批⑭）：装备者摸牌阶段从牌堆末
        // 20 张窗口（pop 摸牌位）中取最大点数牌移至摸牌位（登记
        // 差异上游按牌价值加权随机+任意 drawBegin 收敛为阶段钩）
        if ctx
            .core
            .player(player)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Yuruyi)
            && ctx.core.deck.len() > 1
        {
            let len = ctx.core.deck.len();
            let window = len.min(20);
            let mut best = len - 1;
            for i in (len - window)..len {
                if ctx.catalog.cards[ctx.core.deck[i] as usize].rank
                    > ctx.catalog.cards[ctx.core.deck[best] as usize].rank
                {
                    best = i;
                }
            }
            if best != len - 1 {
                let card = ctx.core.deck.remove(best);
                ctx.core.deck.push(card);
            }
        }
        // 白翼（Boss 少昊，规则 v295）：第 3 轮前敌方摸牌 -1
        //（上游 boss_baiyi_draw phaseDrawBegin num--）。
        let baiyi_cut = ctx.core.turn.round < 3
            && !ctx.core.is_boss_side(player)
            && ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_BAIYI)
                .map(|def| {
                    ctx.core
                        .skills
                        .keys()
                        .any(|(holder, dd)| *dd == def && ctx.core.player(*holder).alive)
                })
                .unwrap_or(false);
        // 灵矢（swd 计亭，swd deferred 消解批 113）：锁定——
        // 装备区有马时摸牌阶段额外摸一张（上游 lingshi_draw
        // phaseDrawBegin num++ 同口径；跳摸路径不进入本臂）
        let lingshi_draw = ctx
            .catalog
            .skill_by_name("lingshi")
            .map(|def| ctx.core.has_skill(player, def))
            .unwrap_or(false)
            && ctx.core.player(player).equips.iter().any(|c| {
                matches!(
                    ctx.catalog.card_slot_of(ctx.core, *c),
                    Some(crate::catalog::EquipSlot::HorsePlus)
                        | Some(crate::catalog::EquipSlot::HorseMinus)
                )
            });
        // 神威（Boss 庞统浴火授予，批 118 规则 v306）：摸牌阶段
        // 多摸 min(3, 其他角色数)（上游 shenwei phaseDrawBegin
        // `trigger.num += Math.min(3, game.players.length-1)`）
        let shenwei_draw = if ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::BOSS_SHENWEI_KEY)
            .map(|def| ctx.core.has_skill(player, def))
            .unwrap_or(false)
        {
            let others = ctx.core.alive_players().filter(|p| *p != player).count();
            others.min(3)
        } else {
            0
        };
        // 灭道（swd 批 152，规则 v321 同批）：摸牌阶段多摸
        // 「已损失体力」张（上游 miedao1 phaseDrawBegin
        // trigger.num += maxHp-hp 同口径）
        let miedao_draw = if ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::SWD_MIEDAO_KEY)
            .map(|def| ctx.core.has_skill(player, def))
            .unwrap_or(false)
        {
            (ctx.core.player(player).max_hp as i32 - ctx.core.player(player).hp).max(0) as usize
        } else {
            0
        };
        // 诈降（L27 批 4，`sbzhaxiang`）：摸牌阶段额外摸「已损失
        // 体力」张（上游 sbzhaxiang_draw：trigger.num +=
        // getDamagedHp()，已损失体力 > 0 时）
        let sbzhaxiang_draw = if ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::L27_SBZHAXIANG_KEY)
            .map(|def| ctx.core.has_skill(player, def))
            .unwrap_or(false)
        {
            (ctx.core.player(player).max_hp as i32 - ctx.core.player(player).hp).max(0) as usize
        } else {
            0
        };
        // 摸牌翻倍（L20 批 1，`_xigua_mopaifanbei_`）：摸牌 +2
        //（上游配置 mpfbmoshi 门控；原生以实例启用承载）
        let mopai_extra = usize::from(
            ctx.catalog
                .skill_by_name(crate::vocabulary::host::L20_MOPAI_KEY)
                .map(|def| ctx.core.has_skill(player, def))
                .unwrap_or(false),
        ) * 2;
        // 脉轮（swd 批 137，规则 v316）：5=少摸一张、6=多摸一张
        //（上游 mailun5/6 phaseDrawBegin trigger.num∓1 同口径）
        let mailun_draw: i32 = match ctx.core.player(player).mailun_mode {
            5 => -1,
            6 => 1,
            _ => 0,
        };
        // 刺骨寒霜（gwent，第 938 轮）：上游 `phaseDrawBegin` 时 `trigger.num--`
        // ⇒ 摸牌数 -1（`event.num > 0` 才减，同上游 filter）
        let cigu_cut = i32::from(
            ctx.catalog
                .skill_by_name(crate::vocabulary::host::GW_CIGU_STATE_KEY)
                .map(|def| ctx.core.has_skill(player, def))
                .unwrap_or(false),
        );
        let base_draw =
            (if baiyi_cut { 1i32 } else { 2i32 } + mailun_draw - cigu_cut).max(0) as usize;
        ctx.draw_cards(
            player,
            base_draw
                + usize::from(lingshi_draw)
                + shenwei_draw
                + miedao_draw
                + sbzhaxiang_draw
                + mopai_extra,
        );
        // 拷贝栏（swd deferred 消解批 肯迪）：锁定——摸牌后额
        // 外摸 1 并将一张手牌（确定性首张）置于牌堆顶（登记差
        // 异上游玩家自选；置顶牌留待下位摸牌者）
        if ctx
            .catalog
            .skill_by_name("kbolan")
            .map(|def| ctx.core.has_skill(player, def))
            .unwrap_or(false)
            && !ctx.core.player(player).hand.is_empty()
        {
            ctx.draw_cards(player, 1);
            if let Some(c) = ctx.core.player(player).hand.first().copied() {
                ctx.take_hand_card(player, c);
                ctx.core.deck.push(c);
            }
        }
        // 雪霁冰雹（swd 卡牌面补切批②）：标记角色摸牌阶段摸
        // 牌数 +1（上游 xuejibingbao phaseDrawBegin num++ 同
        // 口径；计数 2→1→0 两轮后移除）
        if ctx.core.player(player).xuejibingbao > 0 {
            ctx.draw_cards(player, 1);
            ctx.core.player_mut(player).xuejibingbao -= 1;
        }
        // 神恩（Boss 太昊/炎帝 光环，规则 v238）：挑战者侧
        // 摸牌 +1（上游 boss_shenen2 phaseDrawBegin num++ 同
        // 口径；光环激活=场上存在神恩持有者）
        {
            let active = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_SHENEN)
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
            .skill_by_name(crate::vocabulary::names::BOSS_WUZANG)
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
            .skill_by_name(crate::vocabulary::names::BOSS_CANGMU)
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
            .skill_by_name(crate::vocabulary::names::BOSS_NIANRUI)
        {
            if ctx.core.has_skill(player, def) {
                ctx.draw_cards(player, 2);
            }
        }
        // 太平（Boss 黑无常，规则 v256）：摸牌阶段 +2（年瑞
        // 同位）
        if let Some(def) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_TAIPING)
        {
            if ctx.core.has_skill(player, def) {
                ctx.draw_cards(player, 2);
            }
        }
    }
}
