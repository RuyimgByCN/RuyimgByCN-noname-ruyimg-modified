//! 翻面提交与弃牌后宿主触发分发（自 `kernel/events.rs` 拆分，行为零变化）。

use super::*;

pub(in crate::kernel) fn commit_turn_over_before(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    cancelled: bool,
) {
    if cancelled {
        return;
    }
    // 凶兽（Boss 模式共有技，规则 v208）：你不能被翻面（上游
    // boss_xiongshou_turn turnOverBefore priority 20 trigger.cancel 同
    // 口径——入口直读跳过，死亡取消型基建同款）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XIONGSHOU)
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
        .skill_by_name(crate::vocabulary::names::BOSS_SHENYI)
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
            .skill_by_name(crate::vocabulary::names::BOSS_XUSHI)
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
pub(in crate::kernel) fn commit_cards_discarded(ctx: &mut Ctx<'_>, player: PlayerId) {
    // 望尘（swd 古月圣，批㉑）：自己弃牌阶段弃置过基本牌（TurnState 旗，
    // 换回合清；判定区/装备区弃置路径亦计入——上游 discardEnd cards 同为
    // 实际弃置集合）
    if let Some(f) = ctx.res.frame(ctx.top_frame_id()) {
        if let EventData::CardsDiscarded { cards, .. } = &f.data {
            if ctx.core.turn.current == player
                && ctx.core.turn.phase == crate::event::Phase::Discard
                && cards
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c).is_basic_kind())
            {
                ctx.core.turn.wangchen_discard_basic = true;
            }
        }
    }
    let names = [
        crate::vocabulary::names::BOSS_DANSHU,
        crate::vocabulary::names::BOSS_JINGJUE,
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
        if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
            break;
        }
        let judged = ctx.draw_judge_card_with_elo(player, crate::kernel::EloJudgePref::RedPositive);
        let (suit, _) = ctx.inline_judge_suit_rank(player, judged);
        let red = matches!(
            suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
        let _ = ctx.settle_inline_judge_card(player, suit, judged);
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
        .skill_by_name(crate::vocabulary::names::BOSS_MOYANY)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == mydef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current != *o)
            .collect();
        if let Some(holder) = holders.first() {
            if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                let judged =
                    ctx.draw_judge_card_with_elo(*holder, crate::kernel::EloJudgePref::RedPositive);
                let (suit, _) = ctx.inline_judge_suit_rank(*holder, judged);
                let red = matches!(
                    suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                let _ = ctx.settle_inline_judge_card(*holder, suit, judged);
                if red {
                    let seats = ctx.core.players.len() as PlayerId;
                    if let Some(t) = (0..seats).find(|&t| {
                        t != *holder
                            && ctx.core.player(t).alive
                            && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(*holder)
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
    }
    // 绝策（Boss 马面，规则 v258，一将包李儒旧绝策）：自己回合内其他角
    // 色失去最后一手牌（弃后空手）→ 对其造成 1 点伤害（chooseBool AI→
    // 恒发动确定性近似登记；lose/equip/gain 全事件族→CardsDiscarded 面
    // 登记差异）
    if let Some(jcdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JUECE)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == jcdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current == *o && *o != player)
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
        .skill_by_name(crate::vocabulary::names::BOSS_XIANYIN)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == xydef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current != *o)
            .collect();
        if let Some(holder) = holders.first() {
            if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                let judged =
                    ctx.draw_judge_card_with_elo(*holder, crate::kernel::EloJudgePref::RedPositive);
                let (suit, _) = ctx.inline_judge_suit_rank(*holder, judged);
                let red = matches!(
                    suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                let _ = ctx.settle_inline_judge_card(*holder, suit, judged);
                if red {
                    let seats = ctx.core.players.len() as PlayerId;
                    if let Some(t) = (0..seats).find(|&t| t != *holder && ctx.core.player(t).alive) {
                        ctx.emit(EventData::LoseHp {
                            player: t,
                            amount: 1,
                        });
                    }
                }
            }
        }
    }
    // 伤逝（规则 v260）：失去牌后补牌
    shangshix_draw(ctx, player);
}
