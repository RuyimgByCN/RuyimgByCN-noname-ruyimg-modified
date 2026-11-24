use super::*;

// R05-F2 行数门禁拆分：回合/阶段帧族（TurnBegin/TurnEndWindow/TurnEnded
// 提交、交接与新回合重置）外移 events/turn.rs——行为与可见性零变化，
// re-export 兼容既有 `use events::*` 路径。
pub(crate) mod turn;
mod turn_handoff;
pub(crate) use turn_handoff::maybe_abort_ywuhun;
pub(super) use turn::{commit_turn_begin, commit_turn_end_window, commit_turn_ended};
pub use turn::{begin_phase_loop_turn_for, cleanup_interrupted_turn_for_phase_loop};
mod turn_reset;
mod aux_history;
pub use aux_history::record_dddanzhi_card;
pub(super) use aux_history::{
    commit_recover_skills, record_l11_turn_card_used, record_vm_history, zhp_def,
};
mod triggers;
pub(super) use triggers::{commit_cards_discarded, commit_turn_over_before};

/// 枳枉（十殿阎罗五官王，规则 v265）：其他角色获得牌结算后（CardsMoved
/// 面——摸牌阶段摸牌走 CardsDrawn 天然隔离=上游排除 draw 同口径），其
/// 随机弃置一张手牌（随机→首张确定性近似登记；planetarian 彩蛋分支不
/// 落登记）
pub(super) fn commit_cards_moved(ctx: &mut Ctx<'_>, to: PlayerId) {
    if let Some(zwdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ZHIWANG)
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

/// 连环切换提交：翻转横置状态并写回帧载荷。
pub(super) fn commit_linked(ctx: &mut Ctx<'_>, frame_id: EventId, player: PlayerId) {
    let cur = ctx.core.player(player).linked;
    if (!cur && !crate::rvm_bridge::modifiers::can_gain_link(ctx.core, ctx.catalog, player))
        || (cur && !crate::rvm_bridge::modifiers::can_lose_link(ctx.core, ctx.catalog, player))
    {
        return;
    }
    let next = !cur;
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

fn tao_extra_flags_len(core: &crate::state::GameStateCore, user: PlayerId, frame_id: EventId) -> u32 {
    core.vm_aux
        .flags
        .get(&(user, format!("tao_extra_len:{frame_id}")))
        .copied()
        .unwrap_or(0)
}

/// 读取当前 `CardUsed` 帧记下的额外消耗实体载体列表（仅供只读过滤/询问使用）。
pub fn peek_tao_extra_carriers(
    core: &crate::state::GameStateCore,
    user: PlayerId,
    frame_id: EventId,
) -> Vec<crate::ids::CardId> {
    let n = tao_extra_flags_len(core, user, frame_id);
    (0..n)
        .filter_map(|i| core.vm_aux.flags.get(&(user, format!("tao_extra:{frame_id}:{i}"))).copied())
        .collect()
}

/// 取出并清理当前 `CardUsed` 帧记下的额外消耗实体载体列表。
pub fn take_tao_extra_carriers(
    core: &mut crate::state::GameStateCore,
    user: PlayerId,
    frame_id: EventId,
) -> Vec<crate::ids::CardId> {
    let Some(n) = core.vm_aux.flags.remove(&(user, format!("tao_extra_len:{frame_id}"))) else {
        return Vec::new();
    };
    (0..n)
        .filter_map(|i| core.vm_aux.flags.remove(&(user, format!("tao_extra:{frame_id}:{i}"))))
        .collect()
}

/// 出牌提交：牌离开手牌区。杀/桃/酒等置桌面等待结算。
/// 猴子偷桃（E03）使用前取消写回发生在 AfterTriggers 窗。
pub(super) fn commit_card_used(ctx: &mut Ctx<'_>, mut u: crate::event::CardUseData) {
    let frame_id = ctx.current_frame_id();
    let is_dying_save = ctx
        .core
        .vm_aux
        .flags
        .contains_key(&(u.player, format!("dying_save_frame:{frame_id}")));
    let pre_counted = ctx
        .core
        .vm_aux
        .flags
        .remove(&(u.player, "monkey_pre_counted".to_string()))
        .is_some();
    // 奇佐副本载体槽（Boss 郭嘉，规则 v259）：任一非载体卡牌提交即清
    //（副本用牌自身的提交保留槽位，供其 AfterTriggers 过滤拒递归）
    if ctx.core.turn.qizuo_copy != u.card {
        ctx.core.turn.qizuo_copy = None;
    }
    if u.cancelled {
        // 取消痕迹已在猴子机器的 CardsGiven/CardsDiscarded 中观测。
        return;
    }
    if let Some(card) = u.card {
        let effective_kind = u.effective_kind_in(ctx.core, ctx.catalog);
        let in_hand = ctx.core.player(u.player).hand.contains(&card);
        if !ctx.core.table.contains(&card)
            && (u.use_as.is_some_and(|k| k != ctx.catalog.resolved_card_kind(ctx.core, card)) || !in_hand)
        {
            if let Some(spec) = ctx
                .catalog
                .find_player_vm_view_as(
                    ctx.core,
                    u.player,
                    card,
                    !is_dying_save,
                    Some(effective_kind),
                )
                .cloned()
            {
                let extra = spec.consume_and_record(ctx.core, ctx.catalog, u.player, card);
                if spec.cost == Some(crate::rvm_bridge::VmViewAsCost::TurnOver) {
                    ctx.emit(EventData::TurnOverBefore {
                        player: u.player,
                        cancelled: false,
                    });
                }
                if effective_kind == CardKind::Tao || is_dying_save {
                    if !extra.is_empty() {
                        ctx.core.vm_aux.flags.insert(
                            (u.player, format!("tao_extra_len:{frame_id}")),
                            extra.len() as u32,
                        );
                        for (i, c2) in extra.into_iter().enumerate() {
                            if !ctx.core.table.contains(&c2) {
                                ctx.core.table.push(c2);
                            }
                            ctx.core.vm_aux.flags.insert(
                                (u.player, format!("tao_extra:{frame_id}:{i}")),
                                c2,
                            );
                        }
                    }
                } else {
                    for c2 in extra {
                        ctx.to_discard(Some(c2));
                    }
                }
                if spec.zone == crate::rvm_bridge::VmViewAsZone::None {
                    u.card = None;
                    u.use_as = Some(effective_kind);
                    let fu = ctx
                        .res
                        .frame_mut(frame_id)
                        .expect("card_use frame")
                        .card_use_mut();
                    fu.card = None;
                    fu.use_as = Some(effective_kind);
                }
            }
        }
    }
    let effective_kind = u.effective_kind_in(ctx.core, ctx.catalog);
    let nullified = !is_dying_save
        && matches!(
            effective_kind,
            CardKind::Jiu
                | CardKind::Tianxianjiu
                | CardKind::Zhufangshenshi
                | CardKind::Caoyao
                | CardKind::Xuejibingbao
        )
        && dddanzhi_remove_targets(ctx, frame_id, &mut u);
    if let Some(card) = u.card {
        if ctx.core.player(u.player).hand.contains(&card) {
            if ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(u.player, format!("l11c_twkujian_card:{card}")))
            {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, format!("l11c_twkujian_use_resp:{card}")), 1);
            }
            ctx.take_hand_card(u.player, card);
            if ctx.catalog.vm_card_history {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, format!("l11_carduse_lost:{frame_id}")), 1);
            }
        } else if ctx.core.player(u.player).equips.contains(&card) {
            ctx.core.player_mut(u.player).equips.retain(|c| *c != card);
            ctx.core.declared_slots.remove(&card);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, u.player, card);
            ctx.emit(EventData::CardsLost {
                player: u.player,
                cards: vec![card],
                origins: vec![crate::event::LostOrigin::Equip],
            });
        } else if let Some(lost_from_gnjinfan) = {
            let mut hit = None;
            for ((owner, key), pile) in ctx.core.vm_aux.piles.iter_mut() {
                if *owner == u.player {
                    let len_before = pile.len();
                    pile.retain(|&c| c != card);
                    if pile.len() != len_before {
                        hit = Some(key == "gnjinfan");
                        break;
                    }
                }
            }
            hit
        } {
            if lost_from_gnjinfan {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, format!("l11c_gnjinfan_lost:{card}")), 1);
            }
            ctx.emit(EventData::CardsLost {
                player: u.player,
                cards: vec![card],
                origins: vec![crate::event::LostOrigin::Hand],
            });
        } else if ctx.core.table.contains(&card) {
            // 神秘果再结算（swd 卡牌面窗口机批）：原体自弃堆直上桌面（处理
            // 区暂存）后重发射——提交段自桌面取，不经手牌（AfterTriggers 窗
            // 口内手牌数无扰动）；随后按牌种臂照常置桌面/弃置。
            ctx.core.table.retain(|c| *c != card);
        } else if u.use_as.is_some() && ctx.core.discard.contains(&card) {
            ctx.core.discard.retain(|c| *c != card);
        } else if let Some(holder) = u.use_as.and_then(|_| {
            (0..ctx.core.players.len() as PlayerId).find(|&p| ctx.core.player(p).hand.contains(&card))
        }) {
            if ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(holder, format!("l11c_twkujian_card:{card}")))
            {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((holder, format!("l11c_twkujian_use_resp:{card}")), 1);
            }
            ctx.take_hand_card(holder, card);
            if ctx.catalog.vm_card_history {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((holder, format!("l11_carduse_lost:{frame_id}")), 1);
            }
        } else {
            // 保持原 panic 语义（调用方先验证）。
            if ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(u.player, format!("l11c_twkujian_card:{card}")))
            {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, format!("l11c_twkujian_use_resp:{card}")), 1);
            }
            ctx.take_hand_card(u.player, card);
            if ctx.catalog.vm_card_history {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, format!("l11_carduse_lost:{frame_id}")), 1);
            }
        }
    }
    // 全局用牌计数（U05 xianjian 小蛮，规则 v163 灵泉门控；上游
    // getHistory("useCard") 同口径——未被取消的使用全计；猴子前置放行路径已记则跳过）
    if !pre_counted {
        ctx.core.player_mut(u.player).total_used += 1;
        // 沐风（swd 桓远之，批㉑）：本回合已使用基本牌（TurnState 旗，换回合清）
        if u.effective_kind_in(ctx.core, ctx.catalog).is_basic_kind() {
            ctx.core.turn.mufeng_used_basic = true;
        }
    }
    // 邪酬（Boss 相柳，规则 v223）：激活态下其他角色使用【桃】/【酒】
    // → 相柳获 1 蛇毒标记（上游 boss_echou global useCard 同口径）
    let effective_kind = u.effective_kind_in(ctx.core, ctx.catalog);
    if matches!(
        u.use_as.unwrap_or(effective_kind),
        CardKind::Tao | CardKind::Jiu
    ) {
        if let Some(edef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_ECHOU)
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
    // 服毒（十殿阎罗泰山王，规则 v268）：其他角色使用【桃】时，随机另
    // 一名其他角色失去 1 点体力（上游 global useCard boss_fudu；随机→
    // 座位序首个确定性近似登记；候选=全体减持有者减使用者）
    {
        let kind = u.use_as.unwrap_or(effective_kind);
        if kind == CardKind::Tao {
            if let Some(fddef) = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_FUDU)
            {
                let seats = ctx.core.players.len() as PlayerId;
                if seats > 2 {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == fddef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    if let Some(t) = (0..seats).find(|&t| {
                        t != u.player && !holders.contains(&t) && ctx.core.player(t).alive
                    }) {
                        if holders
                            .iter()
                            .any(|h| *h != u.player && ctx.core.player(*h).alive)
                        {
                            ctx.emit(EventData::LoseHp {
                                player: t,
                                amount: 1,
                            });
                        }
                    }
                }
            }
        }
    }
    // 冥顽（Boss 梼杌，规则 v219）：回合内锁定集已激活（非空）时每用
    // 一张牌摸一张（上游 boss_minwan_draw useCard 同口径）
    if ctx.core.turn.current == u.player && !ctx.core.turn.minwan_targets.is_empty() {
        if let Some(mdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_MINWAN)
        {
            if ctx.core.has_skill(u.player, mdef) {
                ctx.start_draw(u.player, 1);
            }
        }
    }
    // 栖梧（群雄乱入 Boss 庞统，规则 v277）：使用梅花牌后若已受伤回复
    // 1 点体力（上游 useCard club+hp<maxHp recover 同口径；事件面与娇孝
    // 臂同族——机器直发 useCard 同覆盖）
    if let Some(qwdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_QIWU)
    {
        let club = matches!(
            crate::kernel::card_use_suit(ctx.core, ctx.catalog, &u),
            Some(crate::catalog::Suit::Club)
        );
        if club
            && ctx.core.has_skill(u.player, qwdef)
            && ctx.core.player(u.player).alive
            && ctx.core.player(u.player).hp > 0
            && ctx.core.player(u.player).hp < ctx.core.player(u.player).max_hp as i32
        {
            ctx.recover(u.player, 1, None);
        }
    }
    // 娇孝（Boss 甄姬，规则 v262）：成为红色牌的目标后摸 1（上游 target
    // useCardToTargeted frequent 同口径；红色=实体花色 Heart/Diamond；
    // 含延时自置目标）
    if let Some(jxdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JIAOXIA)
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
                crate::kernel::card_use_suit(ctx.core, ctx.catalog, &u),
                Some(crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond)
            );
            // 显式目标面：targets 含持有者；闪电自置：targets 空（隐式
            // 自目标，上游 useCardToTargeted target=使用者同口径）
            let is_self_shandian =
                u.targets.is_empty() && u.player == *holder && effective_kind == CardKind::ShanDian;
            if red && (u.targets.contains(holder) || is_self_shandian) {
                ctx.start_draw(*holder, 1);
            }
        }
    }
    // view-as 声明持久化：延时锦囊转化（国色）需跨阶段保存声明牌种，
    // 判定生效时按声明牌种执行（目录牌种不再可靠）。
    if let (Some(card), Some(as_kind)) = (u.card, u.use_as) {
        ctx.core.declared_kinds.insert(card, as_kind);
    }
    let yaoshu_stacks = ctx
        .core
        .vm_aux
        .flags
        .get(&(u.player, "l11_yaoshu_effect_stacks".to_string()))
        .copied()
        .unwrap_or(0);
    if yaoshu_stacks > 0 {
        if yaoshu_stacks == 1 {
            ctx.core
                .vm_aux
                .flags
                .remove(&(u.player, "l11_yaoshu_effect_stacks".to_string()));
        } else {
            ctx.core.vm_aux.flags.insert(
                (u.player, "l11_yaoshu_effect_stacks".to_string()),
                yaoshu_stacks - 1,
            );
        }
        if let Some(card) = u.card {
            ctx.core.table.retain(|&c| c != card);
            if !ctx.core.discard.contains(&card) {
                ctx.to_discard(card);
            }
            ctx.log(LogEvent::CardNullified {
                player: u.player,
                card,
            });
        }
        if let Some(f) = ctx.res.frame_mut(frame_id) {
            let cu = f.card_use_mut();
            cu.cancelled = true;
            cu.targets.clear();
        }
        return;
    }
    if !pre_counted {
        if let Some(card) = u.card {
            ctx.log(LogEvent::CardPlayed {
                card,
                player: u.player,
                targets: u.targets.clone(),
                presentation: None,
            });
        } else {
            ctx.log(LogEvent::VirtualCardPlayed {
                kind: u.effective_kind_in(ctx.core, ctx.catalog),
                player: u.player,
                targets: u.targets.clone(),
                presentation: None,
            });
        }
    }
    // view-as 转化（武圣/龙胆）：以声明牌种结算，而非目录牌种
    let effective = u.effective_kind_in(ctx.core, ctx.catalog);
    match effective {
        CardKind::Tao => {
            ctx.core.table.extend(u.card);
        }
        CardKind::Zhufangshenshi => {
            // 贴房绅视（U05 swd 第十批，card/swd/card.js:1958）：锦囊先置桌面，
            // 经无懈窗口（未抵消→设距免目标）；useCardAfter 摸 1 由 `g_zhufangshenshi`
            // （on_card_use_after）无论是否被无懈皆触发。
            ctx.core.table.extend(u.card);
        }
        CardKind::Caoyao => {
            // 草药（U05 swd 第三批，规则 v87）：为受伤目标回复 1 点体力
            //（目标合法性在 PlayCard 门控：距离 1 + hp<maxHp）
            ctx.to_discard(u.card);
            if !nullified {
                let t = *u.targets.first().expect("caoyao has one target");
                ctx.recover(t, 1, u.card);
            }
        }
        CardKind::Xuejibingbao => {
            // 雪霁冰雹（swd 卡牌面补切批②）：目标标记 2 个摸牌阶段摸牌
            // 数 +1（Draw 钩消费；重标记在 PlayCard 门控拒绝）
            ctx.to_discard(u.card);
            if !nullified {
                let t = *u.targets.first().expect("xuejibingbao has one target");
                ctx.core.player_mut(t).xuejibingbao = 2;
            }
        }
        CardKind::Fengyinzhidan => {
            // 封印之丹（swd 卡牌面补切批②）：随机使用两张普通锦囊牌——
            // 登记差异：原生确定性窄池{五谷→全体其他存活者, 决斗→座位序
            // 首个其他存活者}×2（炮台 v108 随机锦囊最小闭环同口径）
            ctx.to_discard(u.card);
            let others: Vec<PlayerId> = (0..ctx.core.players.len())
                .map(|i| i as PlayerId)
                .filter(|p| *p != u.player && ctx.core.player(*p).alive)
                .collect();
            if others.is_empty() {
                return;
            }
            ctx.use_virtual_card(u.player, CardKind::Wugu, others.clone());
            let others2: Vec<PlayerId> = (0..ctx.core.players.len())
                .map(|i| i as PlayerId)
                .filter(|p| *p != u.player && ctx.core.player(*p).alive)
                .collect();
            if let Some(t) = others2.first() {
                ctx.use_virtual_card(u.player, CardKind::Duel, vec![*t]);
            }
        }
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
            if (ctx.catalog.vm_card_history
                || ctx.catalog.skill_by_name("txhj_fangtianhuaji").is_some())
                && ctx.core.turn.phase == Phase::Play
                && u.player == ctx.core.turn.current
            {
                let prev_cnt = ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(u.player, "l11_phase_sha_count".to_string()))
                    .copied()
                    .unwrap_or(0);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, "l11_phase_sha_count".to_string()), prev_cnt + 1);
                if prev_cnt == 0 {
                    ctx.core.vm_aux.flags.insert(
                        (u.player, format!("l11_first_sha_frame:{frame_id}")),
                        1,
                    );
                }
            }
            // 出牌阶段限用一次（普通/火/雷杀共享）；龙颅山峰地图效果
            //（mtg 首切片，规则 v100）下第二张杀走额外标记——若本次为
            // 加成额外杀则置 land_sha_used，否则置常规已用
            // 破月（拓跋玉儿，swd 武将面批 28）：红杀不计入限一次（上
            // 游 stat 回退同口径）
            let poyue_red_free = matches!(
                crate::kernel::card_use_suit(ctx.core, ctx.catalog, &u),
                Some(crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond)
            ) && ctx
                .catalog
                .skill_by_name("poyue")
                .is_some_and(|def| ctx.core.has_skill(u.player, def));
            // 愚钝·福面（swd 武将面批 82）：仅限愚钝自身转化之杀不计入出杀次数
            //（由 SwdYudun 挂 swd_yudun_sha:{c} 精准界定，不误豁免还侠/妖童/破晓等其他转化杀）
            let yudun_free = u.card.is_some_and(|c| {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(u.player, format!("swd_yudun_sha:{c}")))
                    .is_some()
            });
            let vm_va_free = ctx
                .core
                .vm_aux
                .flags
                .remove(&(u.player, "vm_va_sha_no_count".to_string()))
                .unwrap_or(0)
                != 0;
            let l11_wushen_heart_free = matches!(
                crate::kernel::card_use_suit(ctx.core, ctx.catalog, &u),
                Some(crate::catalog::Suit::Heart)
            ) && ctx.catalog.skill_by_name("wushen").is_some_and(|d| {
                ctx.catalog.skills[d as usize].behavior.as_host_behavior()
                    == Some(crate::vocabulary::host::HostBehavior::L11VmFixA)
                    && ctx.core.has_skill(u.player, d)
                    && !ctx.catalog.is_skill_chanyuan_blocked(ctx.core, u.player, d)
            });
            let qiangwu_mark = ctx
                .core
                .vm_aux
                .flags
                .get(&(u.player, "l11_qiangwu_rank".to_string()))
                .copied()
                .unwrap_or(0);
            let l11_qiangwu_gt_free = ctx
                .core
                .vm_aux
                .flags
                .get(&(u.player, "l11_qiangwu_active".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
                && u.card.is_some_and(|c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| u32::from(cd.rank) > qiangwu_mark)
                });
            if ctx
                .catalog
                .skill_by_name("xiandeng")
                .is_some_and(|def| ctx.core.has_skill(u.player, def))
            {
                let tstamp = crate::rvm_bridge::modifiers::d34_l11_mods_npc_b::turn_stamp(ctx.core);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, "l11c_turn_sha_used_stamp".to_string()), tstamp);
            }
            if ctx.core.turn.phase == Phase::Play && u.player == ctx.core.turn.current {
                if crate::rvm_bridge::modifiers::d34_l11_mods_vm_c::has_unsealed_vm_c_skill(
                    ctx.core,
                    ctx.catalog,
                    u.player,
                    "new_repaoxiao",
                    true,
                ) {
                    let tstamp =
                        crate::rvm_bridge::modifiers::d34_l11_mods_npc_b::turn_stamp(ctx.core);
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((u.player, "l11c_new_repaoxiao2_turn".to_string()), tstamp);
                }
                if ctx
                    .catalog
                    .skill_by_name("jintao")
                    .is_some_and(|def| ctx.core.has_skill(u.player, def))
                {
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((u.player, "q4_play_sha_count".to_string()))
                        .or_insert(0) += 1;
                }
                let qiangwu_no_count = crate::rvm_bridge::modifiers::eval_sha_no_use_count(
                    ctx.core,
                    ctx.catalog,
                    u.player,
                    u.card,
                    u.effective_kind(ctx.catalog),
                );
                if !qiangwu_no_count && !vm_va_free {
                    if crate::rvm_bridge::modifiers::player_tracks_sha_used_count(
                        ctx.core,
                        ctx.catalog,
                        u.player,
                    ) {
                        *ctx.core
                            .vm_aux
                            .flags
                            .entry((u.player, "sha_used_count".to_string()))
                            .or_insert(0) += 1;
                    }
                    if ctx.core.turn.sha_used_play_phase {
                        if ctx.core.current_land == Some(crate::catalog::CardKind::Longlushanfeng)
                            && !vm_va_free
                            && !l11_wushen_heart_free
                            && !l11_qiangwu_gt_free
                        {
                            ctx.core.turn.land_sha_used = true;
                        }
                    } else if !poyue_red_free
                        && !yudun_free
                        && !vm_va_free
                        && !l11_wushen_heart_free
                        && !l11_qiangwu_gt_free
                    {
                        ctx.core.turn.sha_used_play_phase = true;
                    }
                }
            }
            ctx.core.table.extend(u.card);
        }
        CardKind::Jiu | CardKind::Tianxianjiu => {
            turn::commit_jiu_or_tianxianjiu_use(ctx, &u, is_dying_save, nullified);
        }
        CardKind::Shenmiguo => {
            // 神秘果（swd 卡牌面窗口机批）：载体牌直接弃置（不置桌面）
            //——再结算由 CardUsed after 窗口行为发射（mufeng 等基本牌
            // 计数已在前置段按真实牌种覆盖）；清除 yuchankan 临时声明。
            if let Some(c) = u.card {
                ctx.core.declared_kinds.remove(&c);
            }
            ctx.to_discard(u.card);
        }
        // Le/Bing/ShanDian：先置桌面，经无懈窗口（使用时开窗，对齐旧线）
        // 由 PlaceDelayedMachine 放入判定区或弃置。
        _ => {
            if let Some(c) = u.card {
                if !ctx.core.table.contains(&c) {
                    ctx.core.table.push(c);
                }
            }
        }
    }
    // 突击计数 / 回合已用牌数 / 三类计数（猴子前置放行路径已记则跳过）
    if !pre_counted {
        ctx.core.turn.cards_used_turn += 1;
        if !is_dying_save || (ctx.core.turn.phase == Phase::Play && u.player == ctx.core.turn.current)
        {
            ctx.core.turn.cards_used_play_phase += 1;
            if effective.is_basic_trick_or_equip() {
                ctx.core.turn.bte_used_play_phase += 1;
            }
        }
        record_l11_turn_card_used(
            ctx.core,
            ctx.catalog,
            u.player,
            u.card,
            u.suit(ctx.catalog),
            effective,
        );
    }
    if ctx.catalog.vm_card_history
        && !is_dying_save
        && ctx.core.turn.phase == Phase::Play
        && u.player == ctx.core.turn.current
    {
        let mut seq = ctx
            .core
            .vm_aux
            .flags
            .get(&(0, "play_phase_seq".to_string()))
            .copied()
            .unwrap_or(0);
        if seq == 0 {
            seq = 1;
            ctx.core
                .vm_aux
                .flags
                .insert((0, "play_phase_seq".to_string()), seq);
        }
        if seq > 0 {
            let next_idx = ctx
                .core
                .vm_aux
                .flags
                .get(&(0, "play_phase_uses".to_string()))
                .copied()
                .unwrap_or(0)
                + 1;
            ctx.core
                .vm_aux
                .flags
                .insert((0, "play_phase_uses".to_string()), next_idx);
            ctx.core.vm_aux.flags.insert(
                (u.player, format!("card_use_play_seq:{frame_id}")),
                seq,
            );
            ctx.core.vm_aux.flags.insert(
                (u.player, format!("card_use_play_idx:{frame_id}")),
                next_idx,
            );
        }
    }
    if nullified {
        if let Some(card) = u.card {
            ctx.core.table.retain(|&c| c != card);
            if !ctx.core.discard.contains(&card) {
                ctx.to_discard(card);
            }
        }
    }
}

/// 暗织（3D精选 skill.js:5825-5836）：接收者独立持有一次黑色目标免疫。
pub(super) fn dddanzhi_remove_targets(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
    u: &mut crate::event::CardUseData,
) -> bool {
    if !u
        .suit_in(ctx.core, ctx.catalog)
        .is_some_and(|s| matches!(s, crate::catalog::Suit::Spade | crate::catalog::Suit::Club))
    {
        return false;
    }
    let kind = u.effective_kind_in(ctx.core, ctx.catalog);
    let mut targets = match kind {
        CardKind::Wugu => ctx.core.alive_players().collect(),
        CardKind::Taoyuan => ctx
            .core
            .alive_players()
            .filter(|&p| p != u.player)
            .collect(),
        _ if !u.targets.is_empty() => u.targets.clone(),
        CardKind::Tao
        | CardKind::Jiu
        | CardKind::Tianxianjiu
        | CardKind::Wuzhong
        | CardKind::ShanDian => vec![u.player],
        _ => return false,
    };
    let before = targets.len();
    let card = u.card.expect("black card has a physical suit");
    targets.retain(|&target| !consume_dddanzhi_ward(ctx, target, card));
    if targets.len() == before {
        return false;
    }
    u.targets = targets;
    ctx.res
        .frame_mut(frame_id)
        .expect("card use frame")
        .card_use_mut()
        .targets = u.targets.clone();
    if !u.targets.is_empty() && matches!(kind, CardKind::Wugu | CardKind::Taoyuan) {
        ctx.core
            .vm_aux
            .flags
            .insert((u.player, format!("dddanzhi_targets:{frame_id}")), 1);
    }
    u.targets.is_empty() || (kind == CardKind::Jiedao && u.targets.len() < 2)
}

pub(super) fn consume_dddanzhi_ward(
    ctx: &mut Ctx<'_>,
    target: PlayerId,
    card: crate::ids::CardId,
) -> bool {
    if !matches!(
        ctx.catalog.cards[card as usize].suit,
        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
    ) {
        return false;
    }
    let stamp = ctx.core.turn.round.saturating_mul(100) + u32::from(ctx.core.turn.current) + 1;
    let flags = &mut ctx.core.vm_aux.flags;
    let active = flags
        .get(&(target, "dddanzhi_immune".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
        && flags
            .get(&(target, "dddanzhi_stamp".to_string()))
            .copied()
            .unwrap_or(0)
            == stamp;
    if active {
        flags.remove(&(target, "dddanzhi_immune".to_string()));
        flags.remove(&(target, "dddanzhi_stamp".to_string()));
    }
    active
}

/// E03 翻面提交：取消位为真则跳过翻转（漩涡面具等锁定技）；
/// 否则翻转武将牌朝向并发出完成事件。
/// 雷击（Boss 青龙，规则 v234）：打出/使用闪后触发——座位序下一名其
/// 他存活角色确定性判定（铁骑内联口径，无鬼道/天妒改判窗；目标选择以
/// 上游 chooseTarget→座位序首名确定性近似登记）：黑桃 → 其受 2 点雷伤；
/// 梅花 → 持有者回复 1、其受 1 点雷伤；其他无事。
pub(super) fn releiji_on_shan(ctx: &mut Ctx<'_>, holder: PlayerId) {
    let releiji = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_RELEIJI)
        .map(|d| ctx.core.has_skill(holder, d))
        .unwrap_or(false);
    if !releiji {
        return;
    }
    let tgt = ctx.core.next_alive(holder);
    if tgt == holder || !ctx.core.player(tgt).alive {
        return;
    }
    if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
        return;
    }
    let judged = ctx.draw_judge_card_with_elo(holder, crate::kernel::EloJudgePref::Releiji);
    let (suit, _) = ctx.inline_judge_suit_rank(holder, judged);
    let _ = ctx.settle_inline_judge_card(holder, suit, judged);
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
