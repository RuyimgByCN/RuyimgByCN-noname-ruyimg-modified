//! L11（太虚幻境）全部 14 张非武器/非纯防具卡牌固有结算与【诏书】主动效果（`l11_cards.rs`）：
//!
//! 1. `txhj_zhijizhibi`（`extension_card.js:233`）
//! 2. `txhj_yuanjiao`（`extension_card.js:305`）
//! 3. `txhj_yiyi`（`extension_card.js:334`）
//! 4. `txhj_jiedao`（`extension_card.js:704`）
//! 5. `txhj_huoshaolianying`（`extension_card.js:775`）
//! 6. `txhj_diaohu`（`extension_card.js:951`）
//! 7. `txhj_caomu`（`extension_card.js:1076`）
//! 8. `txhj_wangmeizhike`（`extension_card.js:1287`）
//! 9. `txhj_shengdong`（`extension_card.js:1356`）
//! 10. `txhj_shuiyan`（`extension_card.js:1437`）
//! 11. `txhj_jiuchiroulin`（`extension_card.js:1897`）
//! 12. `huogong`（`extension_skill.js:458`）
//! 13. `txzhaoshu_skill` / `txzhaoshu_global`（`extension_card.js:1977-2085`）

use super::l11_cards_asks::*;
use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::kernel::NullifyWindowMachine;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn wait_card_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_card_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        Flow::Done
    }
}

fn orig_card(m: &HostMachine) -> Option<CardId> {
    m.cards.first().copied()
}

fn cleanup_and_done(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(cid) = orig_card(m) {
        finish_table_card(ctx, cid);
    }
    Flow::Done
}

fn cleanup_loop_done(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(cid) = m.card {
        finish_table_card(ctx, cid);
    }
    Flow::Done
}

fn spawn_child_nullify(
    ctx: &mut Ctx<'_>,
    card: CardId,
    source: PlayerId,
    effect: Box<dyn SkillMachine>,
) {
    ctx.res.parks.insert(
        0,
        crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Intrinsic {
                frame: crate::resolution::NO_FRAME,
            },
            machine: Box::new(NullifyWindowMachine::new(
                card,
                source,
                false,
                ctx.core.seat_order_from(source),
                false,
                true,
                false,
                Some(effect),
            )),
            awaiting_answer: false,
        },
    );
}

pub(in crate::content::host) fn try_l11_card_intrinsic(
    ctx: &mut Ctx<'_>,
    u: &CardUseData,
    _effective_kind: CardKind,
) -> Option<Option<Box<dyn SkillMachine>>> {
    let owner = u.player;
    let is_virtual_huogong = u.card.is_none() && flag(ctx.core, owner, VIRTUAL_HUOGONG_FLAG) > 0;
    let virtual_cid_flag = if u.card.is_none() {
        flag(ctx.core, owner, VIRTUAL_TRICK_CID_FLAG)
    } else {
        0
    };
    let key: &str = if is_virtual_huogong {
        set_flag(ctx.core, owner, VIRTUAL_HUOGONG_FLAG, 0);
        "huogong"
    } else if virtual_cid_flag > 0 {
        set_flag(ctx.core, owner, VIRTUAL_TRICK_CID_FLAG, 0);
        crate::content::host::intrinsic::base_card_key(
            ctx.catalog.cards.get((virtual_cid_flag - 1) as usize)?.name,
        )
    } else {
        let card = u.card?;
        let allow_l11c_huogong = u.use_as == Some(CardKind::Linghunzhihuo)
            && flag(ctx.core, owner, &format!("l11c_huogong_use:{card}")) > 0;
        if allow_l11c_huogong {
            set_flag(ctx.core, owner, &format!("l11c_huogong_use:{card}"), 0);
        } else if u.use_as.is_some() {
            return None;
        }
        crate::content::host::intrinsic::base_card_key(
            ctx.catalog.cards.get(card as usize)?.name,
        )
    };

    if !matches!(
        key,
        "txhj_zhijizhibi"
            | "txhj_yuanjiao"
            | "txhj_yiyi"
            | "txhj_jiedao"
            | "txhj_huoshaolianying"
            | "txhj_diaohu"
            | "txhj_caomu"
            | "txhj_wangmeizhike"
            | "txhj_shengdong"
            | "txhj_shuiyan"
            | "txhj_jiuchiroulin"
            | "huogong"
            | "txzhaoshu_skill"
            | "txzhaoshu_global"
    ) {
        return None;
    }

    let wuxie_cid = u.card.unwrap_or_else(|| {
        ctx.catalog
            .card_by_name(key)
            .unwrap_or(0)
    });
    let owns_disposal = u.card.is_some();

    // Multi-target sequential nullify tricks (`txhj_yiyi`, `txhj_jiuchiroulin`):
    // outer parent orchestrates targets cleanly and owns table-card disposal after all targets settle.
    if matches!(key, "txhj_yiyi" | "txhj_jiuchiroulin") {
        let targets = if key == "txhj_yiyi" && u.targets.is_empty() {
            ctx.core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| ctx.core.player(p).alive && ctx.core.is_friend_of(ctx.catalog, owner, p))
                .collect()
        } else {
            u.targets.clone()
        };
        let parent = Box::new(HostMachine {
            def_key: format!("{key}__loop"),
            behavior: HostBehavior::L11TrickCard,
            owner,
            target: None,
            card: u.card,
            cards: Vec::new(),
            targets,
            stage: HostStage::Act,
        });
        return Some(Some(parent));
    }

    let (target, targets) = match key {
        "txhj_jiedao" | "txhj_shengdong" => {
            (u.targets.first().copied(), u.targets.clone())
        }
        "txzhaoshu_skill" | "txzhaoshu_global" => (Some(owner), vec![owner]),
        "txhj_huoshaolianying" => (
            u.targets
                .first()
                .copied()
                .or_else(|| {
                    let next = ctx.core.next_alive(owner);
                    (next != owner && ctx.core.player(next).alive).then_some(next)
                }),
            Vec::new(),
        ),
        "txhj_zhijizhibi" => (u.targets.first().copied(), u.targets.clone()),
        _ => (
            u.targets
                .first()
                .copied()
                .or_else(|| alive_others(ctx.core, owner).into_iter().next()),
            u.targets.clone(),
        ),
    };

    let effect: Box<dyn SkillMachine> = Box::new(HostMachine {
        def_key: key.to_string(),
        behavior: HostBehavior::L11TrickCard,
        owner,
        target,
        card: None,
        cards: u.card.into_iter().collect(),
        targets,
        stage: HostStage::Act,
    });

    Some(Some(Box::new(NullifyWindowMachine::new(
        wuxie_cid,
        owner,
        false,
        ctx.core.seat_order_from(owner),
        false,
        true,
        owns_disposal,
        Some(effect),
    ))))
}

pub(in crate::content::host) fn handle_l11_trick_card(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    if let Some(base) = m.def_key.strip_suffix("__loop") {
        let one_key = match base {
            "txhj_yiyi" => "txhj_yiyi__one".to_string(),
            _ => "txhj_jiuchiroulin__one".to_string(),
        };
        while let Some(next_t) = m.targets.first().copied() {
            m.targets.remove(0);
            if ctx.core.player(next_t).alive {
                let card_id = m.card.unwrap_or(0);
                let cards = if base == "txhj_yiyi" {
                    Vec::new()
                } else {
                    m.card.into_iter().collect()
                };
                let eff = Box::new(HostMachine {
                    def_key: one_key,
                    behavior: HostBehavior::L11TrickCard,
                    owner,
                    target: Some(next_t),
                    card: None,
                    cards,
                    targets: vec![next_t],
                    stage: HostStage::Act,
                });
                spawn_child_nullify(ctx, card_id, owner, eff);
                return Flow::Yield;
            }
        }
        return cleanup_loop_done(m, ctx);
    }
    match m.def_key.as_str() {
        "txhj_yiyi__one" => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            match m.stage {
                HostStage::Act => {
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    m.stage = HostStage::DrawSettle;
                    ctx.start_draw(t, 2);
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let pool = he_cards(ctx.core, t);
                    if pool.len() <= 2 {
                        discard_player_cards(ctx, t, &pool);
                        return Flow::Done;
                    }
                    wait_card_ask(m, ctx, HostStage::SelectCard, ASK_YIYI_DISCARD)
                }
                HostStage::SelectCard => {
                    let chosen = answered_cards(ctx);
                    if !chosen.is_empty() {
                        discard_player_cards(ctx, t, &chosen);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "txhj_huoshaolianying" => {
            let src_card = orig_card(m);
            match m.stage {
                HostStage::Act => {
                    let Some(cur_t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                        return cleanup_and_done(m, ctx);
                    };
                    if m.targets.contains(&cur_t) {
                        return cleanup_and_done(m, ctx);
                    }
                    m.targets.push(cur_t);
                    m.stage = HostStage::DrawSettle;
                    emit_damage(ctx, Some(owner), cur_t, 1, DamageNature::Fire, src_card);
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let Some(prev_t) = m.target else {
                        return cleanup_and_done(m, ctx);
                    };
                    // Chain along consecutive alive friends starting from `prev_t.next` without re-opening NullifyWindow.
                    let next_seat = ctx.core.next_alive(prev_t);
                    if next_seat != prev_t
                        && ctx.core.player(next_seat).alive
                        && ctx.core.is_friend_of(ctx.catalog, prev_t, next_seat)
                        && !m.targets.contains(&next_seat)
                    {
                        m.target = Some(next_seat);
                        m.targets.push(next_seat);
                        emit_damage(ctx, Some(owner), next_seat, 1, DamageNature::Fire, src_card);
                        return Flow::Yield;
                    }
                    cleanup_and_done(m, ctx)
                }
                _ => cleanup_and_done(m, ctx),
            }
        }
        "txhj_jiuchiroulin__one" => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let src_card = orig_card(m);
            match m.stage {
                HostStage::Act => {
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    let has_jiu = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .any(|&c| effective_card_kind(ctx.core, ctx.catalog, c) == CardKind::Jiu);
                    if !has_jiu {
                        m.stage = HostStage::DrawSettle;
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, src_card);
                        return Flow::Yield;
                    }
                    wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_JIUCHIROULIN_RESPOND)
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                            ctx.out.push(LogEvent::CardResponded {
                                player: t,
                                card: c,
                                presentation: None,
                            });
                            ctx.emit(EventData::CardResponded { card: c, player: t });
                            return Flow::Done;
                        }
                    }
                    m.stage = HostStage::DrawSettle;
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, src_card);
                    Flow::Yield
                }
                _ => Flow::Done,
            }
        }
        "txhj_zhijizhibi" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return cleanup_and_done(m, ctx);
                };
                let unseen_in_kingdoms = matches!(
                    ctx.core.mode,
                    crate::state::GameMode::Kingdoms
                ) && ctx.core.player(t).revealed_generals.iter().any(|&r| !r);
                if ctx.core.player(t).hand.is_empty() && !unseen_in_kingdoms {
                    // Upstream `extension_card.js:256`: finishes with 0 draw when controls is empty.
                    return cleanup_and_done(m, ctx);
                }
                set_flag(ctx.core, owner, "l11_zhijizhibi_peeked", 1);
                if !ctx.core.player(t).hand.is_empty() {
                    let h = ctx.core.player(t).hand.clone();
                    ctx.out.push(LogEvent::CardsRevealed {
                        cards: h,
                        to: vec![owner],
                    });
                    return wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_ZHIBI_CONFIRM);
                }
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 1);
                Flow::Yield
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 1);
                Flow::Yield
            }
            HostStage::DrawSettle => cleanup_and_done(m, ctx),
            _ => cleanup_and_done(m, ctx),
        },
        "txhj_yuanjiao" => match m.stage {
            HostStage::Act => {
                m.stage = HostStage::TargetChoice;
                if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    ctx.start_draw(t, 1);
                    return Flow::Yield;
                }
                handle_l11_trick_card(m, ctx)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::DrawSettle;
                let extra = super::l11_common_b4::yuanjiao_draw_bonus(ctx.core, ctx.catalog, owner);
                ctx.start_draw(owner, 3 + extra);
                Flow::Yield
            }
            _ => cleanup_and_done(m, ctx),
        },
        "txhj_jiedao" => {
            let holder = m.targets.first().copied().or(m.target);
            let victim = m.targets.get(1).copied();
            match m.stage {
                HostStage::Act => {
                    let Some(h) = holder.filter(|&p| ctx.core.player(p).alive) else {
                        return cleanup_and_done(m, ctx);
                    };
                    m.target = Some(h);
                    let has_sha = victim.is_some_and(|v| ctx.core.player(v).alive)
                        && ctx
                            .core
                            .player(h)
                            .hand
                            .iter()
                            .any(|&c| effective_card_kind(ctx.core, ctx.catalog, c).is_sha_like());
                    if !has_sha {
                        m.stage = HostStage::DrawSettle;
                        emit_lose_hp(ctx, h, 1);
                        return Flow::Yield;
                    }
                    wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_JIEDAO_USE_SHA)
                }
                HostStage::TargetChoice => {
                    let Some(h) = holder else {
                        return cleanup_and_done(m, ctx);
                    };
                    if let (Some(ChoiceOption::Card(sha_c)), Some(v)) =
                        (ctx.answer().first().copied(), victim.filter(|&p| ctx.core.player(p).alive))
                    {
                        if ctx.core.player(h).hand.contains(&sha_c)
                            && effective_card_kind(ctx.core, ctx.catalog, sha_c).is_sha_like()
                        {
                            ctx.core.player_mut(h).hand.retain(|&c| c != sha_c);
                            ctx.core.table.push(sha_c);
                            ctx.note_cards_lost(h, &[sha_c]);
                            let eff = effective_card_kind(ctx.core, ctx.catalog, sha_c);
                            let use_data = super::sha_card_use_data(ctx, h, sha_c, vec![v], Some(eff));
                            m.stage = HostStage::DrawSettle;
                            ctx.emit(EventData::CardUsed(use_data));
                            return Flow::Yield;
                        }
                    }
                    m.stage = HostStage::DrawSettle;
                    emit_lose_hp(ctx, h, 1);
                    Flow::Yield
                }
                _ => cleanup_and_done(m, ctx),
            }
        }
        "txhj_diaohu" => match m.stage {
            HostStage::Act => {
                if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    ctx.turn_over(t);
                    m.stage = HostStage::TargetChoice;
                    ctx.start_draw(owner, 1);
                    return Flow::Yield;
                }
                cleanup_and_done(m, ctx)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::DrawSettle;
                if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    ctx.start_draw(t, 1);
                    return Flow::Yield;
                }
                cleanup_and_done(m, ctx)
            }
            _ => cleanup_and_done(m, ctx),
        },
        "txhj_caomu" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return cleanup_and_done(m, ctx);
                };
                if !he_cards(ctx.core, t).is_empty() {
                    return wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_CAOMU_DISCARD);
                }
                m.stage = HostStage::TargetChoice;
                m.card = None;
                let near: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(t)
                    .into_iter()
                    .filter(|&p| {
                        p != t
                            && ctx.core.player(p).alive
                            && ctx.core.computed_distance(ctx.catalog, t, p) <= 1
                    })
                    .collect();
                for p in near {
                    draw_direct_cards(ctx, p, 1);
                }
                cleanup_and_done(m, ctx)
            }
            HostStage::TargetChoice => {
                if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    let chosen = answered_cards(ctx);
                    if !chosen.is_empty() {
                        discard_player_cards(ctx, t, &chosen);
                    }
                    let near: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(t)
                        .into_iter()
                        .filter(|&p| {
                            p != t
                                && ctx.core.player(p).alive
                                && ctx.core.computed_distance(ctx.catalog, t, p) <= 1
                        })
                        .collect();
                    for p in near {
                        draw_direct_cards(ctx, p, 1);
                    }
                }
                cleanup_and_done(m, ctx)
            }
            _ => cleanup_and_done(m, ctx),
        },
        "txhj_wangmeizhike" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return cleanup_and_done(m, ctx);
                };
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::OrdinaryJudge {
                    owner: t,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::CompareTargetPick => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return cleanup_and_done(m, ctx);
                };
                let suit_code = flag(ctx.core, t, "last_judge_suit");
                set_flag(ctx.core, t, "last_judge_suit", 0);
                set_flag(ctx.core, t, "last_judge_rank", 0);
                m.stage = HostStage::DrawSettle;
                if suit_code == 3 {
                    ctx.recover(t, 1, orig_card(m));
                    return cleanup_and_done(m, ctx);
                }
                ctx.start_draw(t, 1);
                Flow::Yield
            }
            _ => cleanup_and_done(m, ctx),
        },
        "txhj_shengdong" => {
            let t1 = m.targets.first().copied().or(m.target);
            let t2 = m.targets.get(1).copied();
            match m.stage {
                HostStage::Act => {
                    let Some(t1) = t1.filter(|&p| ctx.core.player(p).alive) else {
                        return cleanup_and_done(m, ctx);
                    };
                    if ctx.core.player(owner).hand.is_empty() {
                        return cleanup_and_done(m, ctx);
                    }
                    m.target = Some(t1);
                    wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_SHENGDONG_GIVE1)
                }
                HostStage::TargetChoice => {
                    let Some(t1) = t1.filter(|&p| ctx.core.player(p).alive) else {
                        return cleanup_and_done(m, ctx);
                    };
                    let chosen = answered_cards(ctx);
                    if chosen.is_empty() {
                        return cleanup_and_done(m, ctx);
                    }
                    move_cards_to_hand(ctx, owner, t1, &chosen);
                    let Some(t2) = t2.filter(|&p| ctx.core.player(p).alive) else {
                        return cleanup_and_done(m, ctx);
                    };
                    let h1 = ctx.core.player(t1).hand.clone();
                    if h1.is_empty() {
                        return cleanup_and_done(m, ctx);
                    }
                    if h1.len() <= 2 {
                        move_cards_to_hand(ctx, t1, t2, &h1);
                        return cleanup_and_done(m, ctx);
                    }
                    m.target = Some(t1);
                    wait_card_ask(m, ctx, HostStage::SelectCard, ASK_SHENGDONG_GIVE2)
                }
                HostStage::SelectCard => {
                    if let (Some(t1), Some(t2)) = (
                        t1.filter(|&p| ctx.core.player(p).alive),
                        t2.filter(|&p| ctx.core.player(p).alive),
                    ) {
                        let chosen = answered_cards(ctx);
                        if !chosen.is_empty() {
                            move_cards_to_hand(ctx, t1, t2, &chosen);
                        }
                    }
                    cleanup_and_done(m, ctx)
                }
                _ => cleanup_and_done(m, ctx),
            }
        }
        "txhj_shuiyan" => {
            let Some(t) = m.target else {
                return cleanup_and_done(m, ctx);
            };
            match m.stage {
                HostStage::Act => {
                    if !ctx.core.player(t).alive {
                        return cleanup_and_done(m, ctx);
                    }
                    let sum: u32 = ctx
                        .core
                        .player(t)
                        .equips
                        .iter()
                        .map(|&c| u32::from(effective_card_rank(ctx.catalog, c)))
                        .sum();
                    if sum < 6 {
                        m.stage = HostStage::DrawSettle;
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, orig_card(m));
                        return Flow::Yield;
                    }
                    wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_SHUIYAN_DISCARD)
                }
                HostStage::TargetChoice => {
                    let chosen = answered_cards(ctx);
                    let rank_sum: u32 = chosen
                        .iter()
                        .map(|&c| u32::from(effective_card_rank(ctx.catalog, c)))
                        .sum();
                    if !chosen.is_empty() && rank_sum >= 6 {
                        discard_player_cards(ctx, t, &chosen);
                        return cleanup_and_done(m, ctx);
                    }
                    if !chosen.is_empty() {
                        // Insufficient rank sum when cards were picked: re-prompt cleanly or cancel
                        return wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_SHUIYAN_DISCARD);
                    }
                    m.stage = HostStage::DrawSettle;
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, orig_card(m));
                    Flow::Yield
                }
                _ => cleanup_and_done(m, ctx),
            }
        }
        "huogong" => {
            let Some(t) = m.target else {
                return cleanup_and_done(m, ctx);
            };
            match m.stage {
                HostStage::Act => {
                    if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                        return cleanup_and_done(m, ctx);
                    }
                    if ol_huoji_active(ctx.core, ctx.catalog, owner) {
                        let len = ctx.core.player(t).hand.len();
                        let idx = ctx.core.rng.gen_range(len as u32) as usize;
                        let show_c = ctx.core.player(t).hand[idx];
                        return finish_huogong_show(m, ctx, owner, t, show_c);
                    }
                    wait_card_ask(m, ctx, HostStage::TargetChoice, ASK_HUOGONG_SHOW)
                }
                HostStage::TargetChoice => {
                    let Some(show_c) = answered_cards(ctx).into_iter().next() else {
                        return cleanup_and_done(m, ctx);
                    };
                    finish_huogong_show(m, ctx, owner, t, show_c)
                }
                HostStage::SelectCard => {
                    if let Some(disc_c) = answered_cards(ctx).into_iter().next() {
                        let show_c = m.cards.get(1).copied();
                        discard_player_cards(ctx, owner, &[disc_c]);
                        if crate::rvm_bridge::modifiers::has_fenchengluetu(
                            ctx.core,
                            ctx.catalog,
                            owner,
                        ) {
                            if let Some(sc) = show_c.filter(|c| {
                                ctx.core.player(t).hand.contains(c) && ctx.core.player(owner).alive
                            }) {
                                move_cards_to_hand(ctx, t, owner, &[sc]);
                            }
                        }
                        m.stage = HostStage::DrawSettle;
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Fire, orig_card(m));
                        return Flow::Yield;
                    }
                    cleanup_and_done(m, ctx)
                }
                _ => cleanup_and_done(m, ctx),
            }
        }
        "txzhaoshu_skill" | "txzhaoshu_global" => {
            if let Some(cid) = orig_card(m) {
                if m.stage == HostStage::Act && m.cards.first().copied() == Some(cid) && ctx.core.table.contains(&cid) {
                    // Card play resolution: move `txzhaoshu` card onto owner's martial zone (`piles`) & activate skills
                    ctx.core.table.retain(|&c| c != cid);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((owner, "txzhaoshu_skill".to_string()))
                        .or_default()
                        .push(cid);
                    ensure_perm_skill(ctx, owner, "txzhaoshu_skill");
                    for seat in alive_all(ctx.core, owner) {
                        ensure_perm_skill(ctx, seat, "txzhaoshu_global");
                    }
                    crate::resolution::sync_trigger_owners(ctx.core, ctx.res);
                    return Flow::Done;
                }
            }
            // Active skill invocation for `txzhaoshu_skill` or `txzhaoshu_global`
            match m.def_key.as_str() {
                "txzhaoshu_skill" => {
                    let ts = turn_stamp(ctx.core);
                    let cur = if flag(ctx.core, owner, "l11_zhaoshu_skill_ts") == ts {
                        flag(ctx.core, owner, "l11_zhaoshu_skill_cnt")
                    } else {
                        0
                    };
                    set_flag(ctx.core, owner, "l11_zhaoshu_skill_ts", ts);
                    set_flag(ctx.core, owner, "l11_zhaoshu_skill_cnt", cur + 1);
                    let ying = ctx
                        .core
                        .vm_aux
                        .piles
                        .remove(&(owner, "txzhaoshu_cards".to_string()))
                        .unwrap_or_default();
                    for c in ying {
                        ctx.to_discard(c);
                    }
                    let mut pool = Vec::new();
                    for (idx, name) in [
                        "zhujinqiyuan",
                        "chuqibuyi",
                        "dongzhuxianji",
                        "shuiyanqijunx",
                    ]
                    .iter()
                    .enumerate()
                    {
                        if flag(ctx.core, 0, &format!("l11_zhaoshu_got:{idx}")) == 0 {
                            pool.push((idx, *name));
                        }
                    }
                    if !pool.is_empty() {
                        let pick_idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let (slot, chosen_name) = pool[pick_idx];
                        set_flag(ctx.core, 0, &format!("l11_zhaoshu_got:{slot}"), 1);
                        let fallback_kind = match slot {
                            0 => CardKind::Dismantle,
                            1 => CardKind::Duel,
                            2 => CardKind::Wuzhong,
                            _ => CardKind::Shuigong,
                        };
                        let found_cid = ctx
                            .core
                            .deck
                            .iter()
                            .chain(ctx.core.discard.iter())
                            .copied()
                            .find(|&c| {
                                ctx.catalog
                                    .cards
                                    .get(c as usize)
                                    .is_some_and(|cd| cd.name == chosen_name)
                            })
                            .or_else(|| {
                                ctx.core
                                    .deck
                                    .iter()
                                    .chain(ctx.core.discard.iter())
                                    .copied()
                                    .find(|&c| ctx.catalog.card_kind(c) == fallback_kind)
                            });
                        if let Some(cid) = found_cid {
                            gain_card_from_deck_or_discard(ctx, owner, cid);
                        } else {
                            draw_direct_cards(ctx, owner, 1);
                        }
                    }
                    Flow::Done
                }
                "txzhaoshu_global" => match m.stage {
                    HostStage::Act => {
                        let holders: Vec<PlayerId> = alive_all(ctx.core, owner)
                            .into_iter()
                            .filter(|&p| {
                                ctx.core.is_friend_of(ctx.catalog, owner, p)
                                    && has_named_skill(ctx.core, ctx.catalog, p, "txzhaoshu_skill")
                            })
                            .collect();
                        if holders.is_empty() || ctx.core.player(owner).hand.is_empty() {
                            refund_once_used(ctx, owner, "txzhaoshu_global");
                            return Flow::Done;
                        }
                        if holders.len() == 1 {
                            m.target = Some(holders[0]);
                            return wait_card_ask(
                                m,
                                ctx,
                                HostStage::SelectCard,
                                ASK_ZHAOSHU_GLOBAL_CARDS,
                            );
                        }
                        wait_card_ask(
                            m,
                            ctx,
                            HostStage::TargetChoice,
                            ASK_ZHAOSHU_GLOBAL_TARGET,
                        )
                    }
                    HostStage::TargetChoice => {
                        let Some(tgt) = answered_players(ctx).into_iter().next() else {
                            refund_once_used(ctx, owner, "txzhaoshu_global");
                            return Flow::Done;
                        };
                        m.target = Some(tgt);
                        wait_card_ask(m, ctx, HostStage::SelectCard, ASK_ZHAOSHU_GLOBAL_CARDS)
                    }
                    HostStage::SelectCard => {
                        let chosen = answered_cards(ctx);
                        let Some(tgt) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                            refund_once_used(ctx, owner, "txzhaoshu_global");
                            return Flow::Done;
                        };
                        if chosen.is_empty() {
                            refund_once_used(ctx, owner, "txzhaoshu_global");
                            return Flow::Done;
                        }
                        for c in &chosen {
                            if ctx.core.player(owner).hand.contains(c) {
                                ctx.core.player_mut(owner).hand.retain(|x| x != c);
                                ctx.note_cards_lost(owner, &[*c]);
                                ctx.core
                                    .vm_aux
                                    .piles
                                    .entry((tgt, "txzhaoshu_cards".to_string()))
                                    .or_default()
                                    .push(*c);
                            }
                        }
                        let ts = turn_stamp(ctx.core);
                        let cur = if flag(ctx.core, owner, "l11_zhaoshu_global_ts") == ts {
                            flag(ctx.core, owner, "l11_zhaoshu_global_cnt")
                        } else {
                            0
                        };
                        set_flag(ctx.core, owner, "l11_zhaoshu_global_ts", ts);
                        set_flag(ctx.core, owner, "l11_zhaoshu_global_cnt", cur + 1);
                        Flow::Done
                    }
                    _ => Flow::Done,
                },
                _ => Flow::Done,
            }
        }
        _ => cleanup_and_done(m, ctx),
    }
}

pub(in crate::content::host) fn activate_l11_zhaoshu(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    let state = ctx.state;
    if state.turn.phase != Phase::Play || !is_unblocked_l11_owner(state, ctx.catalog, owner) {
        return None;
    }
    match skill.name.as_str() {
        "txzhaoshu_skill" => {
            let ts = turn_stamp(state);
            let used = if flag(state, owner, "l11_zhaoshu_skill_ts") == ts {
                flag(state, owner, "l11_zhaoshu_skill_cnt")
            } else {
                0
            };
            if used >= 2 {
                return None;
            }
            let cards = state
                .vm_aux
                .piles
                .get(&(owner, "txzhaoshu_cards".to_string()))?;
            if cards.len() < 2 {
                return None;
            }
            let mut suits = Vec::new();
            for &c in cards {
                let s = ctx.catalog.cards.get(c as usize).map(|cd| cd.suit);
                if let Some(suit) = s {
                    if !suits.contains(&suit) {
                        suits.push(suit);
                    }
                }
            }
            if suits.len() < 2 {
                return None;
            }
            active_skill_machine("txzhaoshu_skill", HostBehavior::L11TrickCard, owner)
        }
        "txzhaoshu_global" => {
            let ts = turn_stamp(state);
            let used = if flag(state, owner, "l11_zhaoshu_global_ts") == ts {
                flag(state, owner, "l11_zhaoshu_global_cnt")
            } else {
                0
            };
            if used >= 2 || state.player(owner).hand.is_empty() {
                return None;
            }
            let has_holder = alive_all(state, owner).into_iter().any(|p| {
                state.is_friend_of(ctx.catalog, owner, p)
                    && has_named_skill(state, ctx.catalog, p, "txzhaoshu_skill")
            });
            if !has_holder {
                return None;
            }
            active_skill_machine("txzhaoshu_global", HostBehavior::L11TrickCard, owner)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn trigger_meta_l11_trick(name: &str) -> Option<TriggerMeta> {
    match name {
        "txzhaoshu_skill" | "txzhaoshu_global" => Some(TriggerMeta {
            event: EventKind::TurnEnded,
            timing: Timing::Normal,
            priority: 1,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_trick(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn filter_l11_trick(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
    _event: &EventData,
) -> bool {
    false
}

pub(super) fn ol_huoji_active(
    core: &crate::state::GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    flag(core, owner, "l11_olhuoji_active") > 0
        || crate::rvm_bridge::modifiers::d34_l11_mods_npc_a::has_npc_a_skill_unsealed(
            core, catalog, owner, "olhuoji",
        )
}

fn finish_huogong_show(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    t: PlayerId,
    show_c: CardId,
) -> Flow {
    ctx.out.push(LogEvent::CardsRevealed {
        cards: vec![show_c],
        to: Vec::new(),
    });
    if m.cards.len() < 2 {
        m.cards.push(show_c);
    } else {
        m.cards[1] = show_c;
    }
    let show_suit = ctx.catalog.effective_card_suit(ctx.core, t, show_c);
    let ol_huoji = ol_huoji_active(ctx.core, ctx.catalog, owner);
    let use_color =
        ol_huoji || crate::rvm_bridge::modifiers::has_wolongmice(ctx.core, ctx.catalog, owner);
    let show_red = matches!(show_suit, Suit::Heart | Suit::Diamond);
    let pool = if ol_huoji {
        he_cards(ctx.core, owner)
    } else {
        ctx.core.player(owner).hand.clone()
    };
    let has_match = pool.into_iter().any(|c| {
        let s = ctx.catalog.effective_card_suit(ctx.core, owner, c);
        if use_color {
            matches!(s, Suit::Heart | Suit::Diamond) == show_red
        } else {
            s == show_suit
        }
    });
    if !has_match {
        return cleanup_and_done(m, ctx);
    }
    wait_card_ask(m, ctx, HostStage::SelectCard, ASK_HUOGONG_DISCARD)
}
