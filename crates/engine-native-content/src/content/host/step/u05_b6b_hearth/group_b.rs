//! Batch U05-6b (`shahai-hearth` Part 2) Group B: 25 技能 (`fenlie` .. `oldmalymowang`)。

use super::asks::{
    ASK_B6B_BIMENG_CONFIRM, ASK_B6B_BIRI_DISCARD, ASK_B6B_FUSHI_TARGET,
    ASK_B6B_MALYMOWANG_DISCOVER, ASK_B6B_OLDFUSHI_TARGET, ASK_B6B_QIANGHUA_CONFIRM,
    ASK_B6B_SHIXU_DISCARD, clear_b6b_ask, wait_b6b,
};
use super::{
    b6b_flag, clear_b6b_flag, hs_b6b_cnt, hs_b6b_inc, set_b6b_flag, spawn_random_dream_card,
};
use super::super::u05_hearth::{
    alive_others, base_card_name, discard_cards_from_zone, exact_base_name, gain_card_to_hand,
    grant_skill, has_any_skill, is_delayed_trick_hearth, is_hearth_enemy, is_hearth_unique_equip,
    meta, rng_pick, set_flag, spawn_card_by_base,
};
use super::super::{HostMachine, HostSkillBehavior, HostStage};
use crate::archive::kind_from_code;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_b(name: &str) -> Option<TriggerMeta> {
    match name {
        "fenlie" | "oldfenlie" => Some(meta(
            EventKind::CardsDrawn,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardsMoved],
        )),
        "nianfu" | "bingdong" | "luoshi" => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()))
        }
        "xiaorong" | "shixu" | "stuxi" | "jiaohui" | "bimeng" | "liehun" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        "jixuan" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, -50, false, Vec::new())),
        "biri" => Some(meta(EventKind::CardUsed, Timing::Normal, 15, false, Vec::new())),
        "ronghuo" => Some(meta(EventKind::CardUsed, Timing::Normal, 7, false, Vec::new())),
        "zhoujiang" => Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())),
        "malymowang" | "oldmalymowang" => Some(meta(
            EventKind::DamageBefore,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin],
        )),
        // `qianghua` 由 `b6b_on_card_use_after` 手动推机；`stuxi2` 由 `b6b_on_draw_begin` 消费；
        // `qianghua2`/`ronghuo2`/`moyao`/`xjumo` 为被动或空占位。
        _ => None,
    }
}

pub(super) fn limits_b(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn has_active_b(name: &str) -> bool {
    matches!(name, "fushi" | "oldfushi")
}

fn non_unique_gain_cards(
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match event {
        EventData::CardsDrawn { player, cards } => {
            *player == owner
                && cards.iter().any(|&c| !is_hearth_unique_equip(ctx.catalog, c))
        }
        EventData::CardsMoved { to, cards, .. } => {
            *to == owner
                && cards.iter().any(|&c| !is_hearth_unique_equip(ctx.catalog, c))
        }
        _ => false,
    }
}

pub(super) fn filter_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    match skill.name.as_str() {
        "fenlie" => hs_b6b_cnt(st, owner, "fenlie") < 2 && non_unique_gain_cards(ctx, owner, event),
        "oldfenlie" => non_unique_gain_cards(ctx, owner, event),
        "nianfu" => match event {
            EventData::Damage(d) => {
                if d.source == Some(owner) {
                    d.target != owner && !st.player(d.target).equips.is_empty()
                } else if d.target == owner {
                    d.source.is_some_and(|s| {
                        s != owner && st.player(s).alive && !st.player(s).equips.is_empty()
                    })
                } else {
                    false
                }
            }
            _ => false,
        },
        "xiaorong" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner
                    && st.player(owner)
                        .hand
                        .iter()
                        .any(|&c| ctx.catalog.card_kind(c).is_equip())
        ),
        "shixu" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if b6b_flag(st, *player, "hs_shixu_duration") > 3000
                    && st.player(*player).alive
                    && (!st.player(*player).hand.is_empty()
                        || !st.player(*player).equips.is_empty())
        ),
        "jixuan" => matches!(
            event,
            EventData::TurnEndWindow { player, after: true }
                if *player == owner && b6b_flag(st, owner, "hs_active_extra_turn_source") != 1
        ),
        "biri" => match event {
            EventData::CardUsed(u) => {
                u.player != owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && u.targets.len() == 1
                    && u.targets[0] != owner
                    && st.player(u.targets[0]).alive
                    && st.computed_distance(ctx.catalog, owner, u.targets[0]) <= 1
                    && st.player(owner)
                        .hand
                        .iter()
                        .any(|&c| ctx.catalog.card_kind(c) == CardKind::Shan)
            }
            _ => false,
        },
        "stuxi" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner
                    && alive_others(st, owner).into_iter().any(|e| {
                        is_hearth_enemy(st, ctx.catalog, owner, e)
                            && !has_any_skill(st, ctx.catalog, e, "stuxi2")
                    })
        ),
        "bingdong" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && hs_b6b_cnt(st, owner, "bingdong") == 0
        ),
        "luoshi" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && (!st.player(owner).hand.is_empty()
                        || !st.player(owner).equips.is_empty()
                        || d.source.is_some_and(|s| {
                            st.player(s).alive
                                && (!st.player(s).hand.is_empty()
                                    || !st.player(s).equips.is_empty())
                        }))
        ),
        "ronghuo" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && u.nature == DamageNature::Normal
        ),
        "jiaohui" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner && b6b_flag(st, owner, "hs_turn_dealt_damage") == 0
        ),
        "bimeng" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner
        ),
        "zhoujiang" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                let eff = u.effective_kind(ctx.catalog);
                !eff.is_basic_kind() && eff.equip_slot().is_none() && !is_delayed_trick_hearth(eff)
            }
            _ => false,
        },
        "liehun" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner
                    && st.player(owner).hand.iter().any(|&c| {
                        !ctx.catalog.card_kind(c).is_basic_kind()
                            && !is_hearth_unique_equip(ctx.catalog, c)
                    })
        ),
        "malymowang" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.from_chain
                    && hs_b6b_cnt(st, owner, "malymowang") == 0
                    && d.card.is_some_and(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none() && !is_delayed_trick_hearth(k)
                    })
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "oldmalymowang" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.from_chain
                    && d.card.is_some_and(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none() && !is_delayed_trick_hearth(k)
                    })
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn activate_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let st = ctx.state;
    let injured_exists = st
        .alive_players()
        .any(|p| st.player(p).hp < st.player(p).max_hp as i32);
    match skill.name.as_str() {
        "fushi" => Some(hs_b6b_cnt(st, owner, "fushi") == 0 && injured_exists),
        "oldfushi" => Some(injured_exists),
        _ => None,
    }
}

fn is_standard_wuzhong_name(raw: &str) -> bool {
    let exact = exact_base_name(raw);
    exact == "wuzhong" || exact == "无中生有" || exact == "hs_wuzhong"
}

fn spawn_detached_copy_for(
    ctx: &mut Ctx<'_>,
    src_cid: Option<CardId>,
    kind: CardKind,
) -> Option<CardId> {
    let cat = ctx.catalog;
    if let Some(cid) = src_cid {
        let raw = cat.cards[cid as usize].name;
        let exact = exact_base_name(raw);
        if let Some(pos) = ctx
            .core
            .detached_cards
            .iter()
            .position(|&(c, _)| exact_base_name(cat.cards[c as usize].name) == exact)
        {
            return Some(ctx.core.detached_cards.remove(pos).0);
        }
        // `shandianjian`、`hsfashu_*` 等复用 `CardKind::Wuzhong` 的炉石具名牌与 `shihuawuqi`
        // (`CardKind::Tao`) 绝不可跨牌名按 `CardKind` 回退成任意他牌。
        if exact == "shihuawuqi" || (kind == CardKind::Wuzhong && !is_standard_wuzhong_name(raw)) {
            return None;
        }
    }
    if let Some(pos) = ctx.core.detached_cards.iter().position(|&(c, _)| {
        let cname = cat.cards[c as usize].name;
        let cex = exact_base_name(cname);
        cat.card_kind(c) == kind
            && cex != "du"
            && cex != "shihuawuqi"
            && (kind != CardKind::Wuzhong || is_standard_wuzhong_name(cname))
    }) {
        return Some(ctx.core.detached_cards.remove(pos).0);
    }
    None
}

fn spawn_detached_copy_only(ctx: &mut Ctx<'_>, src_cid: CardId) -> Option<CardId> {
    let kind = ctx.catalog.card_kind(src_cid);
    spawn_detached_copy_for(ctx, Some(src_cid), kind)
}

fn step_fenlie(m: &mut HostMachine, ctx: &mut Ctx<'_>, old: bool) -> Flow {
    let sk = if old { "oldfenlie" } else { "fenlie" };
    if b6b_flag(ctx.core, m.owner, "hs_fenlie_firing") > 0 {
        return Flow::Done;
    }
    if !old && hs_b6b_cnt(ctx.core, m.owner, sk) >= 2 {
        return Flow::Done;
    }
    let mut rev = ctx.res.stack.iter().rev();
    let f0 = rev.next();
    let f1 = rev.next();
    let f2 = rev.next();
    let f3 = rev.next();
    let parent_is_draw_phase = matches!(f1.map(|f| &f.data), Some(EventData::DrawPhase { .. }))
        || (matches!(f0.map(|f| &f.data), Some(EventData::CardsDrawn { .. }))
            && matches!(f1.map(|f| &f.data), Some(EventData::DrawGain { .. }))
            && matches!(f2.map(|f| &f.data), Some(EventData::Draw { .. }))
            && matches!(f3.map(|f| &f.data), Some(EventData::DrawPhase { .. })))
        || (matches!(f0.map(|f| &f.data), Some(EventData::CardsDrawn { .. }))
            && matches!(f1.map(|f| &f.data), Some(EventData::Draw { .. }))
            && matches!(f2.map(|f| &f.data), Some(EventData::DrawPhase { .. })));
    if parent_is_draw_phase {
        return Flow::Done;
    }
    let copies_src: Vec<CardId> = m
        .cards
        .iter()
        .copied()
        .filter(|&c| !is_hearth_unique_equip(ctx.catalog, c))
        .collect();
    if copies_src.is_empty() {
        return Flow::Done;
    }
    if !old {
        hs_b6b_inc(ctx.core, m.owner, sk);
    }
    set_b6b_flag(ctx.core, m.owner, "hs_fenlie_firing", 1);
    for c in copies_src {
        if let Some(copy) = spawn_detached_copy_only(ctx, c) {
            gain_card_to_hand(ctx, m.owner, copy);
        }
    }
    clear_b6b_flag(ctx.core, m.owner, "hs_fenlie_firing");
    Flow::Done
}

fn step_malymowang(m: &mut HostMachine, ctx: &mut Ctx<'_>, limited: bool) -> Flow {
    if ctx.current_damage().is_ok() {
        if !limited || hs_b6b_cnt(ctx.core, m.owner, "malymowang") == 0 {
            if limited {
                hs_b6b_inc(ctx.core, m.owner, "malymowang");
            }
            ctx.modify_damage_amount(1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            let cat = ctx.catalog;
            let mut pool: Vec<CardId> = ctx
                .core
                .detached_cards
                .iter()
                .filter_map(|&(c, _)| {
                    let k = cat.card_kind(c);
                    (!k.is_basic_kind()
                        && k.equip_slot().is_none()
                        && !is_delayed_trick_hearth(k)
                        && exact_base_name(cat.cards[c as usize].name) != "du")
                    .then_some(c)
                })
                .collect();
            let mut picks = Vec::new();
            for _ in 0..3 {
                if let Some(c) = super::super::u05_hearth::rng_pick_mut(ctx, &mut pool) {
                    picks.push(c);
                }
            }
            if picks.is_empty() {
                return Flow::Done;
            }
            m.cards = picks;
            wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_MALYMOWANG_DISCOVER)
        }
        HostStage::SelectCard => {
            clear_b6b_ask(m, ctx, HostStage::SelectCard);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                ctx.core.detached_cards.retain(|&(x, _)| x != cid);
                gain_card_to_hand(ctx, m.owner, cid);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "fenlie" => step_fenlie(m, ctx, false),
        "oldfenlie" => step_fenlie(m, ctx, true),
        "nianfu" => {
            let Ok(d) = ctx.current_damage() else {
                return Flow::Done;
            };
            let other = if d.source == Some(m.owner) {
                if d.target == m.owner {
                    return Flow::Done;
                }
                d.target
            } else if d.target == m.owner {
                let Some(s) = d.source else {
                    return Flow::Done;
                };
                if s == m.owner || !ctx.core.player(s).alive {
                    return Flow::Done;
                }
                s
            } else {
                return Flow::Done;
            };
            let eqs = ctx.core.player(other).equips.clone();
            if let Some(c) = rng_pick(ctx, &eqs) {
                ctx.take_owned_card(other, c);
                gain_card_to_hand(ctx, m.owner, c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: other,
                    to: m.owner,
                });
            }
            Flow::Done
        }
        "xiaorong" => {
            let hand_eqs: Vec<CardId> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.card_kind(c).is_equip())
                .collect();
            if hand_eqs.is_empty() {
                return Flow::Done;
            }
            let n = hand_eqs.len() as u32;
            for c in hand_eqs {
                // `lose(cards)._triggered = null`（skill.js:4845）：不经 `CardsLost` 触发直接置入弃牌堆。
                let hand = &mut ctx.core.player_mut(m.owner).hand;
                if let Some(pos) = hand.iter().position(|&x| x == c) {
                    hand.remove(pos);
                }
                ctx.to_discard(c);
            }
            for _ in 0..(n * 2) {
                let cat = ctx.catalog;
                let basics: Vec<CardId> = ctx
                    .core
                    .detached_cards
                    .iter()
                    .filter_map(|&(c, _)| {
                        let cname = cat.cards[c as usize].name;
                        (cat.card_kind(c).is_basic_kind()
                            && exact_base_name(cname) != "du"
                            && exact_base_name(cname) != "shihuawuqi")
                        .then_some(c)
                    })
                    .collect();
                if let Some(pick) = rng_pick(ctx, &basics) {
                    ctx.core.detached_cards.retain(|&(x, _)| x != pick);
                    gain_card_to_hand(ctx, m.owner, pick);
                } else {
                    // 衍生供应池耗尽后，可复用弃牌堆中已结算的炉石基本牌衍生物（不破坏牌数守恒）。
                    let recycled: Vec<CardId> = ctx
                        .core
                        .discard
                        .iter()
                        .copied()
                        .filter(|&c| {
                            let def = &cat.cards[c as usize];
                            def.off_deck
                                && def.name.starts_with("hs_")
                                && def.kind.is_basic_kind()
                                && exact_base_name(def.name) != "du"
                        })
                        .collect();
                    if let Some(pick) = rng_pick(ctx, &recycled) {
                        ctx.core.discard.retain(|&x| x != pick);
                        gain_card_to_hand(ctx, m.owner, pick);
                    }
                }
            }
            ctx.recover(m.owner, n, None);
            Flow::Done
        }
        "shixu" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                let ms = b6b_flag(ctx.core, t, "hs_shixu_duration");
                clear_b6b_flag(ctx.core, t, "hs_shixu_duration");
                if ms <= 3000 || !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let he_len = ctx.core.player(t).hand.len() + ctx.core.player(t).equips.len();
                if he_len == 0 {
                    return Flow::Done;
                }
                let want = ((ms / 3000) as usize).min(he_len);
                m.card = Some(want as u32);
                wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_SHIXU_DISCARD)
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(t) = m.target {
                    let chosen: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if !chosen.is_empty() {
                        discard_cards_from_zone(ctx, t, &chosen);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: chosen,
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jixuan" => {
            if b6b_flag(ctx.core, m.owner, "hs_active_extra_turn_source") == 1 {
                return Flow::Done;
            }
            ctx.start_draw(m.owner, 1);
            set_b6b_flag(ctx.core, m.owner, "extra_turn_insert_queued", 1);
            set_b6b_flag(ctx.core, m.owner, "hs_queued_extra_turn_source", 1);
            Flow::Done
        }
        "qianghua" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_QIANGHUA_CONFIRM),
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                let eff_code = b6b_flag(ctx.core, m.owner, "hs_qianghua_eff_kind");
                let nat_code = b6b_flag(ctx.core, m.owner, "hs_qianghua_nature");
                clear_b6b_flag(ctx.core, m.owner, "hs_qianghua_eff_kind");
                clear_b6b_flag(ctx.core, m.owner, "hs_qianghua_nature");
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                if hs_b6b_cnt(ctx.core, m.owner, "qianghua") > 0 || !ctx.core.player(m.owner).alive
                {
                    return Flow::Done;
                }
                let kind = kind_from_code(eff_code as u16)
                    .ok()
                    .or_else(|| m.card.map(|c| ctx.catalog.card_kind(c)))
                    .unwrap_or(CardKind::Sha);
                let is_shihuawuqi = m.card.is_some_and(|c| {
                    base_card_name(ctx.catalog.cards[c as usize].name) == "shihuawuqi"
                });
                if kind == CardKind::Tao
                    && !is_shihuawuqi
                    && ctx.core.player(m.owner).hp >= ctx.core.player(m.owner).max_hp as i32
                {
                    return Flow::Done;
                }
                let nature = match nat_code {
                    1 => DamageNature::Fire,
                    2 => DamageNature::Thunder,
                    _ => super::super::u05_hearth::nature_for_kind(kind),
                };
                let valid_targets: Vec<PlayerId> = if !m.targets.is_empty() {
                    let ts: Vec<PlayerId> = m
                        .targets
                        .iter()
                        .copied()
                        .filter(|&t| {
                            ctx.core.player(t).alive
                                && m.card.is_none_or(|src_cid| {
                                    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                        ctx.core, ctx.catalog, m.owner, t, src_cid, kind, nature,
                                    )
                                })
                        })
                        .collect();
                    if ts.is_empty() {
                        return Flow::Done;
                    }
                    ts
                } else {
                    Vec::new()
                };
                hs_b6b_inc(ctx.core, m.owner, "qianghua");
                set_b6b_flag(ctx.core, m.owner, "hs_qianghua_firing", 1);
                if kind.is_sha_like() {
                    set_b6b_flag(ctx.core, m.owner, "vm_va_sha_no_count", 1);
                }
                if let Some(copy) = spawn_detached_copy_for(ctx, m.card, kind) {
                    // 复刻牌进处理区而非手牌（skill.js:4979-4980），防止结算后滞留手牌。
                    ctx.core.table.push(copy);
                    let use_as = (ctx.catalog.card_kind(copy) != kind).then_some(kind);
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: Some(copy),
                        player: m.owner,
                        targets: valid_targets,
                        nature,
                        use_as,
                        cancelled: false,
                    }));
                } else {
                    if let Some(src_cid) = m.card {
                        set_flag(ctx.core, m.owner, "hs_virtual_card_src", src_cid + 1);
                    }
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: m.owner,
                        targets: valid_targets,
                        nature,
                        use_as: Some(kind),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "biri" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_BIRI_DISCARD),
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(ChoiceOption::Card(shan_cid)) = ctx.answer().first().copied() {
                    if ctx.core.player(m.owner).hand.contains(&shan_cid)
                        && ctx.current_card_use().is_ok()
                    {
                        let (used_card, user) = {
                            let u = ctx.current_card_use_mut();
                            u.cancelled = true;
                            (u.card, u.player)
                        };
                        ctx.discard_from_hand(m.owner, shan_cid);
                        if let Some(cid) = used_card {
                            if ctx.core.player(user).hand.contains(&cid) {
                                ctx.take_hand_card(user, cid);
                            }
                            ctx.core.table.retain(|&x| x != cid);
                            ctx.to_discard(cid);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "stuxi" => {
            let pool: Vec<PlayerId> = alive_others(ctx.core, m.owner)
                .into_iter()
                .filter(|&e| {
                    is_hearth_enemy(ctx.core, ctx.catalog, m.owner, e)
                        && !has_any_skill(ctx.core, ctx.catalog, e, "stuxi2")
                })
                .collect();
            if let Some(t) = rng_pick(ctx, &pool) {
                grant_skill(ctx, t, "stuxi2", false);
            }
            Flow::Done
        }
        "bingdong" => {
            if hs_b6b_cnt(ctx.core, m.owner, "bingdong") == 0 {
                hs_b6b_inc(ctx.core, m.owner, "bingdong");
                if let Some(c) = spawn_card_by_base(ctx, "hslingjian_jinjilengdong") {
                    gain_card_to_hand(ctx, m.owner, c);
                }
            }
            Flow::Done
        }
        "luoshi" => {
            let src = ctx.current_damage().ok().and_then(|d| d.source);
            let owner_he: Vec<CardId> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .chain(ctx.core.player(m.owner).equips.iter())
                .copied()
                .collect();
            if let Some(c) = rng_pick(ctx, &owner_he) {
                super::super::u05_hearth::discard_card_from_zone(ctx, m.owner, c);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: vec![c],
                });
            }
            if let Some(s) = src {
                if ctx.core.player(s).alive {
                    let s_he: Vec<CardId> = ctx
                        .core
                        .player(s)
                        .hand
                        .iter()
                        .chain(ctx.core.player(s).equips.iter())
                        .copied()
                        .collect();
                    if let Some(c) = rng_pick(ctx, &s_he) {
                        super::super::u05_hearth::discard_card_from_zone(ctx, s, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: s,
                            cards: vec![c],
                        });
                    }
                }
            }
            Flow::Done
        }
        "ronghuo" => {
            if ctx.current_card_use().is_ok() {
                ctx.current_card_use_mut().nature = DamageNature::Fire;
            }
            Flow::Done
        }
        "fushi" | "oldfushi" => {
            let limited = m.def_key == "fushi";
            let ask_id = if limited {
                ASK_B6B_FUSHI_TARGET
            } else {
                ASK_B6B_OLDFUSHI_TARGET
            };
            match m.stage {
                HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ask_id),
                HostStage::TargetChoice => {
                    clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).alive {
                            if limited {
                                hs_b6b_inc(ctx.core, m.owner, "fushi");
                            }
                            ctx.lose_max_hp(t, 1);
                            if ctx.core.player(t).alive
                                && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
                            {
                                ctx.recover(t, 1, None);
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "jiaohui" => match m.stage {
            HostStage::Act => {
                wait_b6b(m, ctx, HostStage::TargetChoice, super::asks::ASK_B6B_JIAOHUI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                m.target = Some(t);
                wait_b6b(m, ctx, HostStage::SelectNature, super::asks::ASK_B6B_JIAOHUI_CHOICE)
            }
            HostStage::SelectNature => {
                clear_b6b_ask(m, ctx, HostStage::SelectNature);
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(1))
                        if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 =>
                    {
                        ctx.recover(t, 1, None);
                    }
                    _ => ctx.start_draw(t, 1),
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bimeng" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_BIMENG_CONFIRM),
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    spawn_random_dream_card(ctx, m.owner);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhoujiang" => {
            const POOL: [&str; 5] = [
                "hszuzhou_nvwudeganguo",
                "hszuzhou_nvwudepingguo",
                "hszuzhou_nvwudexuetu",
                "hszuzhou_wushushike",
                "hszuzhou_guhuo",
            ];
            let avail: Vec<&str> = POOL
                .iter()
                .copied()
                .filter(|&name| {
                    ctx.core.detached_cards.iter().any(|&(c, _)| {
                        exact_base_name(ctx.catalog.cards[c as usize].name) == name
                    })
                })
                .collect();
            if let Some(pick) = rng_pick(ctx, &avail) {
                if let Some(c) = spawn_card_by_base(ctx, pick) {
                    gain_card_to_hand(ctx, m.owner, c);
                }
            }
            Flow::Done
        }
        "liehun" => {
            let hand_non_basic: Vec<CardId> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    !ctx.catalog.card_kind(c).is_basic_kind()
                        && !is_hearth_unique_equip(ctx.catalog, c)
                })
                .collect();
            for c in hand_non_basic {
                if let Some(copy) = spawn_detached_copy_only(ctx, c) {
                    gain_card_to_hand(ctx, m.owner, copy);
                }
            }
            Flow::Done
        }
        "malymowang" => step_malymowang(m, ctx, true),
        "oldmalymowang" => step_malymowang(m, ctx, false),
        _ => Flow::Done,
    }
}
