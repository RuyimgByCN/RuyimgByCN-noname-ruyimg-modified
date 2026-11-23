//! U05 仙剑 Batch D（9 主动/转换技）：`lingdi`, `xjzongyu`, `dujiang`,
//! `yanzhan`, `feixia`, `huxi`, `guijiang`, `duxinshu`, `tuoqiao`。

use super::xianjian_batch_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{CardUseData, DamageNature, DetachedKind, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(super) fn has_active(name: &str) -> bool {
    matches!(
        name,
        "lingdi" | "xjzongyu" | "dujiang" | "yanzhan" | "feixia" | "huxi" | "guijiang" | "duxinshu"
    )
}

pub(super) fn trigger_meta(name: &str) -> Option<TriggerMeta> {
    match name {
        "tuoqiao" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        _ => None,
    }
}

pub(super) fn filter(skill: &HostSkillBehavior, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
    if ctx.catalog.blocks_all_cards(ctx.state, owner) || ctx.catalog.is_tieji_sealed(ctx.state, owner) {
        return false;
    }
    match skill.name.as_str() {
        "tuoqiao" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: true }
            if *player != owner
                && ctx.state.player(owner).alive
                && ctx.state.player(*player).alive
                && !has_skill_named(ctx.state, ctx.catalog, *player, crate::vocabulary::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
                && hand_and_equips(ctx.state, owner)
                    .into_iter()
                    .any(|c| {
                        is_black_card(ctx.state, ctx.catalog, owner, c)
                            && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                ctx.state,
                                ctx.catalog,
                                owner,
                                *player,
                                c,
                                CardKind::Shihuifen,
                                DamageNature::Normal,
                            )
                    })
        ),
        _ => false,
    }
}

pub(super) fn machine(skill: &HostSkillBehavior, owner: PlayerId, event: &EventData) -> HostMachine {
    let target = match (skill.name.as_str(), event) {
        ("tuoqiao", EventData::Phase { player, phase: Phase::Play, begin: true }) => Some(*player),
        _ => None,
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(super) fn limits(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn can_activate(skill: &HostSkillBehavior, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    if ctx.catalog.blocks_all_cards(ctx.state, owner) || ctx.catalog.is_tieji_sealed(ctx.state, owner) {
        return false;
    }
    match skill.name.as_str() {
        "lingdi" => {
            let need = 1 + flag(ctx.state, owner, "xj_lingdi_uses_turn");
            let mask = flag(ctx.state, owner, "xj_lingdi_suits_mask");
            let has_target = alive_others(ctx.state, owner)
                .into_iter()
                .any(|p| ctx.state.computed_distance(ctx.catalog, owner, p).max(1) == need);
            has_target
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| (mask & suit_bit(ctx.catalog.effective_card_suit(ctx.state, owner, c))) == 0)
        }
        "xjzongyu" => {
            !has_once_turn(ctx.state, owner, "xjzongyu")
                && hand_and_equips(ctx.state, owner)
                    .into_iter()
                    .any(|c| is_black_card(ctx.state, ctx.catalog, owner, c))
        }
        "dujiang" => {
            !has_once_turn(ctx.state, owner, "dujiang")
                && hand_and_equips(ctx.state, owner)
                    .into_iter()
                    .any(|c| is_black_card(ctx.state, ctx.catalog, owner, c))
        }
        "yanzhan" => {
            !has_once_turn(ctx.state, owner, "yanzhan")
                && can_owner_play_sha(ctx.state, ctx.catalog, owner)
                && hand_and_equips(ctx.state, owner)
                    .into_iter()
                    .any(|c| is_red_card(ctx.state, ctx.catalog, owner, c))
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| sha_target_legal(ctx.state, ctx.catalog, owner, p, true))
        }
        "feixia" => {
            !has_once_turn(ctx.state, owner, "feixia")
                && hand_and_equips(ctx.state, owner)
                    .into_iter()
                    .any(|c| is_red_card(ctx.state, ctx.catalog, owner, c))
        }
        "huxi" => {
            can_owner_play_sha(ctx.state, ctx.catalog, owner)
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| sha_target_legal(ctx.state, ctx.catalog, owner, p, true))
        }
        "guijiang" => {
            !has_once_turn(ctx.state, owner, "guijiang")
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| is_black_card(ctx.state, ctx.catalog, owner, c))
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| !has_skill_named(ctx.state, ctx.catalog, p, "guijiang2") && flag(ctx.state, p, "u05_mod:guijiang2") == 0)
        }
        "duxinshu" => {
            !has_once_turn(ctx.state, owner, "duxinshu")
                && alive_others(ctx.state, owner).into_iter().any(|p| !ctx.state.player(p).hand.is_empty())
        }
        _ => false,
    }
}

pub(super) fn activate(skill: &HostSkillBehavior, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<HostMachine> {
    if !can_activate(skill, ctx, owner) {
        return None;
    }
    Some(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05XianjianBatchD,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "lingdi" => step_lingdi(m, ctx, owner),
        "xjzongyu" => step_xjzongyu(m, ctx, owner),
        "dujiang" => step_dujiang(m, ctx, owner),
        "yanzhan" => step_yanzhan(m, ctx, owner),
        "feixia" => step_feixia(m, ctx, owner),
        "huxi" => step_huxi(m, ctx, owner),
        "guijiang" => step_guijiang(m, ctx, owner),
        "duxinshu" => step_duxinshu(m, ctx, owner),
        "tuoqiao" => step_tuoqiao(m, ctx, owner),
        _ => Flow::Done,
    }
}

fn step_lingdi(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_LINGDI_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let mask = flag(ctx.core, owner, "xj_lingdi_suits_mask");
            let suit_b = suit_bit(ctx.catalog.effective_card_suit(ctx.core, owner, c));
            if !ctx.core.player(owner).hand.contains(&c) || (mask & suit_b) != 0 {
                return Flow::Done;
            }
            m.cards = vec![c];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_LINGDI_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let Some(c) = m.cards.pop() else {
                return Flow::Done;
            };
            let need = 1 + flag(ctx.core, owner, "xj_lingdi_uses_turn");
            if t == owner
                || !ctx.core.player(t).alive
                || ctx.core.computed_distance(ctx.catalog, owner, t).max(1) != need
                || !ctx.core.player(owner).hand.contains(&c)
            {
                return Flow::Done;
            }
            let mask = flag(ctx.core, owner, "xj_lingdi_suits_mask");
            let suit_b = suit_bit(ctx.catalog.effective_card_suit(ctx.core, owner, c));
            ctx.discard_from_hand(owner, c);
            set_flag(ctx.core, owner, "xj_lingdi_suits_mask", mask | suit_b);
            set_flag(ctx.core, owner, "xj_lingdi_uses_turn", need);
            ctx.start_draw(owner, 1);
            ctx.start_draw(t, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xjzongyu(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XJZONGYU_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !hand_and_equips(ctx.core, owner).contains(&c) || !is_black_card(ctx.core, ctx.catalog, owner, c) {
                return Flow::Done;
            }
            mark_once_turn(ctx.core, owner, "xjzongyu");
            discard_from_he(ctx, owner, c);
            let mut enemies = alive_enemies(ctx.core, owner);
            if enemies.is_empty() {
                return Flow::Done;
            }
            let mut picked = Vec::new();
            while !enemies.is_empty() && picked.len() < 2 {
                let i = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
                picked.push(enemies.remove(i));
            }
            let order = ctx.core.seat_order_from(owner);
            picked.sort_by_key(|p| order.iter().position(|x| x == p).unwrap_or(usize::MAX));
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets: picked,
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Feibiao),
                cancelled: false,
            }));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dujiang(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_DUJIANG_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !hand_and_equips(ctx.core, owner).contains(&c) || !is_black_card(ctx.core, ctx.catalog, owner, c) {
                return Flow::Done;
            }
            mark_once_turn(ctx.core, owner, "dujiang");
            discard_from_he(ctx, owner, c);
            let enemies = alive_enemies(ctx.core, owner);
            if enemies.is_empty() {
                return Flow::Done;
            }
            let target = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
            if let Some(poison) = take_poison_card(ctx.core, ctx.catalog) {
                set_flag(ctx.core, target, &format!("is_du_card:{poison}"), 1);
                set_flag(ctx.core, target, &format!("xj_du_holder:{poison}"), 1);
                ctx.core.player_mut(target).hand.push(poison);
                ctx.log(LogEvent::CardsGiven {
                    from: owner,
                    to: target,
                    cards: vec![poison],
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn take_poison_card(core: &mut GameStateCore, catalog: &crate::catalog::Catalog) -> Option<CardId> {
    let is_named_du = |c: CardId, allow_fallback: bool| {
        catalog.cards.get(c as usize).is_some_and(|d| {
            d.name == "du" || d.name == "毒" || (allow_fallback && d.name.starts_with("xj_du_"))
        })
    };
    for allow_fallback in [false, true] {
        if let Some(pos) = core.detached_cards.iter().rposition(|&(c, kind)| {
            kind == DetachedKind::Unclaimed && is_named_du(c, allow_fallback)
        }) {
            let (c, _) = core.detached_cards.remove(pos);
            return Some(c);
        }
        if let Some(pos) = core.discard.iter().rposition(|&c| is_named_du(c, allow_fallback)) {
            return Some(core.discard.remove(pos));
        }
        if let Some(pos) = core.deck.iter().rposition(|&c| is_named_du(c, allow_fallback)) {
            return Some(core.deck.remove(pos));
        }
    }
    None
}

fn step_yanzhan(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YANZHAN_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !hand_and_equips(ctx.core, owner).contains(&c) || !is_red_card(ctx.core, ctx.catalog, owner, c) {
                return Flow::Done;
            }
            m.cards = vec![c];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_YANZHAN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let Some(c) = m.cards.pop() else {
                return Flow::Done;
            };
            if !hand_and_equips(ctx.core, owner).contains(&c)
                || !is_red_card(ctx.core, ctx.catalog, owner, c)
                || !sha_target_legal(ctx.core, ctx.catalog, owner, t, true)
            {
                return Flow::Done;
            }
            mark_once_turn(ctx.core, owner, "yanzhan");
            if ctx.core.turn.sha_used_play_phase {
                let extra = flag(ctx.core, owner, "vm_sha_extra_used") + 1;
                set_flag(ctx.core, owner, "vm_sha_extra_used", extra);
            }
            ctx.core.turn.cards_used_play_phase += 1;
            let st = suit_tag(ctx.catalog.effective_card_suit(ctx.core, owner, c));
            set_flag(ctx.core, owner, &format!("xj_yanzhan_card:{c}"), st);
            set_flag(ctx.core, owner, &format!("xj_yanzhan_src_card:{c}"), 1);
            ensure_temp_skill(ctx, t, "yanzhan3", true);
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(c),
                player: owner,
                targets: vec![t],
                nature: DamageNature::Fire,
                use_as: Some(CardKind::FireSha),
                cancelled: false,
            }));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_feixia(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_FEIXIA_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !hand_and_equips(ctx.core, owner).contains(&c) || !is_red_card(ctx.core, ctx.catalog, owner, c) {
                return Flow::Done;
            }
            mark_once_turn(ctx.core, owner, "feixia");
            discard_from_he(ctx, owner, c);
            let enemies: Vec<PlayerId> = alive_enemies(ctx.core, owner)
                .into_iter()
                .filter(|&p| sha_target_legal(ctx.core, ctx.catalog, owner, p, false))
                .collect();
            if !enemies.is_empty() {
                let target = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
                set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![target],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_huxi(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_HUXI_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !sha_target_legal(ctx.core, ctx.catalog, owner, t, true) {
                return Flow::Done;
            }
            m.target = Some(t);
            m.stage = HostStage::DrawSettle;
            m.card = None;
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            let cur = i32::from(ctx.core.player(owner).hujia);
            ctx.core.player_mut(owner).hujia = (cur + 1).min(99) as u8;
            if ctx.core.turn.sha_used_play_phase {
                let extra = flag(ctx.core, owner, "vm_sha_extra_used") + 1;
                set_flag(ctx.core, owner, "vm_sha_extra_used", extra);
            }
            ctx.core.turn.cards_used_play_phase += 1;
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets: vec![t],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Sha),
                cancelled: false,
            }));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_guijiang(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_GUIJIANG_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c) || !is_black_card(ctx.core, ctx.catalog, owner, c) {
                return Flow::Done;
            }
            m.cards = vec![c];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_GUIJIANG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let Some(c) = m.cards.pop() else {
                return Flow::Done;
            };
            if t == owner || !ctx.core.player(t).alive || !ctx.core.player(owner).hand.contains(&c) {
                return Flow::Done;
            }
            mark_once_turn(ctx.core, owner, "guijiang");
            ctx.discard_from_hand(owner, c);
            set_flag(ctx.core, t, "u05_mod:guijiang2", 1);
            set_flag(ctx.core, t, "xj_guijiang2_src", u32::from(owner) + 1);
            ensure_temp_skill(ctx, t, "guijiang2", false);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_duxinshu(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_DUXINSHU_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if t == owner || !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            mark_once_turn(ctx.core, owner, "duxinshu");
            if ctx.core.player(owner).hand.is_empty() {
                return Flow::Done;
            }
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::CardsAsk, ASK_DUXINSHU_STEAL)
        }
        HostStage::CardsAsk => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(c_tgt) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).hand.contains(&c_tgt) || ctx.core.player(owner).hand.is_empty() {
                return Flow::Done;
            }
            m.cards = vec![c_tgt];
            wait_ask(m, ctx, HostStage::SelectCard, ASK_DUXINSHU_GIVE)
        }
        HostStage::SelectCard => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(c_tgt) = m.cards.pop() else {
                return Flow::Done;
            };
            let Some(c_own) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).hand.contains(&c_tgt) || !ctx.core.player(owner).hand.contains(&c_own) {
                return Flow::Done;
            }
            let migrate_du = |core: &mut GameStateCore, from: PlayerId, to: PlayerId, cid: CardId| {
                for prefix in ["is_du_card", "xj_du_holder"] {
                    let key = format!("{prefix}:{cid}");
                    if flag(core, from, &key) > 0 {
                        set_flag(core, from, &key, 0);
                        set_flag(core, to, &key, 1);
                    }
                }
            };
            ctx.give_hand_card_private(t, owner, c_tgt);
            migrate_du(ctx.core, t, owner, c_tgt);
            ctx.give_hand_card_private(owner, t, c_own);
            migrate_du(ctx.core, owner, t, c_own);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_tuoqiao(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TUOQIAO_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !hand_and_equips(ctx.core, owner).contains(&c)
                || !is_black_card(ctx.core, ctx.catalog, owner, c)
            {
                return Flow::Done;
            }
            if let Some(t) = m.target {
                return finish_tuoqiao(ctx, owner, c, t);
            }
            m.cards = vec![c];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_TUOQIAO_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let Some(c) = m.cards.pop() else {
                return Flow::Done;
            };
            finish_tuoqiao(ctx, owner, c, t)
        }
        _ => Flow::Done,
    }
}

fn finish_tuoqiao(ctx: &mut Ctx<'_>, owner: PlayerId, c: CardId, t: PlayerId) -> Flow {
    if t == owner
        || !hand_and_equips(ctx.core, owner).contains(&c)
        || !ctx.core.player(t).alive
        || has_skill_named(ctx.core, ctx.catalog, t, crate::vocabulary::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
        || crate::rvm_bridge::modifiers::eval_card_target_canceled(
            ctx.core,
            ctx.catalog,
            owner,
            t,
            c,
            CardKind::Shihuifen,
            DamageNature::Normal,
        )
    {
        return Flow::Done;
    }
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(c),
        player: owner,
        targets: vec![t],
        nature: DamageNature::Normal,
        use_as: Some(CardKind::Shihuifen),
        cancelled: false,
    }));
    Flow::Done
}
