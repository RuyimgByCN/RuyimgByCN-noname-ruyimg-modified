//! L25 祖安武将 Batch G 询问构造与前半技能步进（技能 1–8：`ssr_lianhuan` 至 `ssr_zhiyan`）。

use super::l25_batch_g_ext::handle_l25_batch_g_ext;
use super::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn spec_ask(
    chooser: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    (!options.is_empty() && options.len() >= min).then_some((
        chooser,
        ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        },
    ))
}

pub(super) fn confirm_ask(chooser: PlayerId, prompt: &'static str) -> Option<(PlayerId, ChoiceSpec)> {
    Some((
        chooser,
        ChoiceSpec {
            kind: ChoiceKind::Confirm,
            prompt,
            options: vec![
                ChoiceOption::Confirm,
                ChoiceOption::Pass,
                ChoiceOption::Tag(1),
                ChoiceOption::Tag(0),
            ],
            min: 1,
            max: 1,
            allow_cancel: true,
        },
    ))
}

pub(super) fn tag_ask(
    chooser: PlayerId,
    prompt: &'static str,
    tags: impl IntoIterator<Item = u8>,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::PickTag,
        prompt,
        tags.into_iter().map(ChoiceOption::Tag).collect(),
        1,
        1,
        allow_cancel,
    )
}

pub(super) fn player_ask(
    chooser: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectPlayers,
        prompt,
        players.into_iter().map(ChoiceOption::Player).collect(),
        min,
        max,
        allow_cancel,
    )
}

pub(super) fn card_ask(
    chooser: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(
        chooser,
        ChoiceKind::SelectCards,
        prompt,
        cards.into_iter().map(ChoiceOption::Card).collect(),
        min,
        max,
        allow_cancel,
    )
}

pub(super) fn is_confirmed(ctx: &Ctx<'_>) -> bool {
    matches!(
        ctx.answer().first().copied(),
        Some(ChoiceOption::Confirm) | Some(ChoiceOption::Tag(1))
    )
}

pub(super) fn pop_top_deck_cards(ctx: &mut Ctx<'_>, count: usize) -> Vec<CardId> {
    let mut out = Vec::new();
    for _ in 0..count {
        if let Some(c) = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop()) {
            out.push(c);
        } else {
            break;
        }
    }
    out
}

pub(super) fn judge_one_card(ctx: &mut Ctx<'_>) -> Option<CardId> {
    let cid = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop())?;
    ctx.out.push(LogEvent::CardsRevealed {
        cards: vec![cid],
        to: Vec::new(),
    });
    ctx.to_discard(cid);
    Some(cid)
}

pub(super) fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_LIANHUAN_DISCARD => card_ask(
            owner,
            "ssr_lianhuan:discard_card",
            lianhuan_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_LIANHUAN_TARGETS => {
            let cands = alive_all(core, owner);
            let max = cands.len().clamp(1, 3);
            player_ask(owner, "ssr_lianhuan:choose_targets", cands, 1, max, false)
        }
        ASK_NEWSANCE_MODE => tag_ask(owner, "ssr_newsance:choose_mode", 0..=2, true),
        ASK_NEWSANCE_TARGET => player_ask(
            owner,
            "ssr_newsance:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_NEWSANCE_CARD => card_ask(
            owner,
            "ssr_newsance:choose_card",
            cards.to_vec(),
            1,
            1,
            false,
        ),
        ASK_NEWXIEBIAN_MODE => tag_ask(owner, "ssr_newxiebian:choose_mode", 0..=3, true),
        ASK_XUNXUN_CARDS => {
            let n = cards.len().clamp(1, 2);
            card_ask(owner, "ssr_xunxun:choose_cards", cards.to_vec(), n, n, false)
        }
        ASK_WANGXI_CARD => card_ask(
            owner,
            "ssr_wangxi:choose_card",
            cards.to_vec(),
            1,
            1,
            true,
        ),
        ASK_JIELIANG_DISCARD => card_ask(
            owner,
            "ssr_jieliang:discard_card",
            jieliang_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_JIELIANG_TARGET => player_ask(
            owner,
            "ssr_jieliang:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_FUTU_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_futu:choose_target", cands, 1, 1, true)
        }
        ASK_FUTU_PINDIAN_OWNER => card_ask(
            owner,
            "ssr_futu:pindian_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_FUTU_PINDIAN_TARGET => card_ask(
            tgt,
            "ssr_futu:target_pindian_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_FUTU_MODE => tag_ask(owner, "ssr_futu:choose_mode", 0..=1, false),
        ASK_DIMENG_TARGETS => player_ask(
            owner,
            "ssr_dimeng:choose_targets",
            alive_others(core, owner),
            2,
            2,
            true,
        ),
        ASK_FUHUI_CONFIRM => confirm_ask(owner, "ssr_fuhui:confirm"),
        ASK_FUHUI_PINDIAN_OWNER => card_ask(
            owner,
            "ssr_fuhui:pindian_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_FUHUI_PINDIAN_TARGET => card_ask(
            tgt,
            "ssr_fuhui:target_pindian_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_ZHIJIAN_GIVE => card_ask(
            owner,
            "ssr_zhijian:give_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_ZHIJIAN_TARGET => player_ask(
            owner,
            "ssr_zhijian:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_GUZHENG_CONFIRM => confirm_ask(owner, "ssr_guzheng:confirm"),
        ASK_GUZHENG_CARD => card_ask(
            owner,
            "ssr_guzheng:choose_card",
            cards.to_vec(),
            1,
            1,
            false,
        ),
        ASK_TIANLEI_TARGET => player_ask(
            owner,
            "ssr_tianlei:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_GAIMING_CARD => card_ask(
            owner,
            "ssr_gaiming:choose_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_NEWGUIDAO_CARD => card_ask(
            owner,
            "ssr_newguidao:choose_card",
            newguidao_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_NEWGUIDAO_TARGET => player_ask(
            owner,
            "ssr_newguidao:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_ZHUJI_DISCARD => card_ask(
            tgt,
            "ssr_zhuji:discard_card",
            he_cards(core, tgt),
            1,
            1,
            true,
        ),
        ASK_DIHUO_TARGET => player_ask(
            owner,
            "ssr_dihuo:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_GUIFU_CARD => card_ask(
            owner,
            "ssr_guifu:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_GUIFU_TARGET => player_ask(
            owner,
            "ssr_guifu:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_RENHONG_TARGETS => {
            let cands = alive_others(core, owner);
            let max = cands.len().clamp(1, 2);
            player_ask(owner, "ssr_renhong:choose_targets", cands, 1, max, true)
        }
        ASK_FANGTONG_SHOW => card_ask(
            owner,
            "ssr_fangtong:show_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_FANGTONG_TARGET => player_ask(
            owner,
            "ssr_fangtong:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_FANGTONG_RESPONSE => {
            let max_rank = flag(core, owner, "l25_fangtong_shown_rank").max(1) as u8;
            let pool: Vec<CardId> = core
                .player(tgt)
                .hand
                .iter()
                .copied()
                .filter(|&c| card_rank(catalog, c) <= max_rank)
                .collect();
            card_ask(tgt, "ssr_fangtong:target_response", pool, 1, 1, true)
        }
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    set_flag(ctx.core, m.owner, "l25_batch_g_ask_id", ask_id);
    if let Some((observer, spec)) = build_ask(
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
        set_flag(ctx.core, m.owner, "l25_batch_g_ask_id", 0);
        Flow::Done
    }
}

pub fn ask_l25_batch_g(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle || stage == HostStage::CardsAsk {
        return None;
    }
    let ask_id = card
        .filter(|&id| id > 0 && id < 40)
        .unwrap_or_else(|| flag(core, owner, "l25_batch_g_ask_id"));
    let (observer, spec) =
        build_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn handle_l25_batch_g_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::CardsAsk
        && ctx.res.answer.as_ref().is_some_and(|a| a.is_empty())
    {
        if m.def_key == "ssr_fangtong" && m.stage == HostStage::ComparePlayerPick {
            return handle_l25_batch_g_ext(m, ctx);
        }
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ssr_lianhuan" => step_ssr_lianhuan(m, ctx),
        "ssr_newsance" => step_ssr_newsance(m, ctx),
        "ssr_newxiebian" => step_ssr_newxiebian(m, ctx),
        "ssr_motu" => step_ssr_motu(m, ctx),
        "ssr_xunxun" => step_ssr_xunxun(m, ctx),
        "ssr_jieliang" => step_ssr_jieliang(m, ctx),
        _ => handle_l25_batch_g_ext(m, ctx),
    }
}

fn step_ssr_lianhuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(102) && m.stage == HostStage::Act {
        let user = m.target.unwrap_or(owner);
        let mut unlinked = false;
        let mut primary = None;
        for &t in &m.targets {
            if ctx.core.player(t).alive
                && (ctx.core.player(t).linked || flag(ctx.core, t, "l25_lianhuan_relink") > 0)
            {
                ctx.core.player_mut(t).linked = false;
                set_flag(ctx.core, t, "l25_lianhuan_relink", 0);
                set_flag(ctx.core, t, "mark:ssr_lianhuan_relink", 0);
                unlinked = true;
                if primary.is_none() {
                    primary = Some(t);
                }
            }
        }
        if unlinked {
            set_flag(ctx.core, owner, "l25_lianhuan_draw", 1);
            ctx.start_draw(owner, 1);
            if let Some(pt) = primary {
                let other_linked = alive_all(ctx.core, owner)
                    .into_iter()
                    .find(|&p| {
                        p != pt
                            && (ctx.core.player(p).linked
                                || flag(ctx.core, p, "l25_lianhuan_relink") > 0)
                    });
                if let Some(copied) = other_linked {
                    ctx.core.player_mut(copied).linked = false;
                    set_flag(ctx.core, copied, "l25_lianhuan_relink", 0);
                    set_flag(ctx.core, copied, "mark:ssr_lianhuan_relink", 0);
                    set_flag(ctx.core, owner, "l25_lianhuan_copied", 1);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(user),
                        copied,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                }
            }
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_LIANHUAN_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_lianhuan");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_LIANHUAN_TARGETS)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if chosen.is_empty() {
                refund_once_used(ctx, owner, "ssr_lianhuan");
                return Flow::Done;
            }
            let disc = std::mem::take(&mut m.cards);
            discard_he_cards(ctx, owner, &disc);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_lianhuan");
            set_flag(ctx.core, owner, "l25_lianhuan_used", cnt);
            let mut unlinked_count = 0u32;
            for p in chosen {
                if !ctx.core.player(p).alive {
                    continue;
                }
                let was_linked = ctx.core.player(p).linked;
                ctx.core.player_mut(p).linked = !was_linked;
                let now_val = u32::from(!was_linked);
                set_flag(ctx.core, p, "l25_lianhuan_relink", now_val);
                set_flag(ctx.core, p, "mark:ssr_lianhuan_relink", now_val);
                if was_linked {
                    unlinked_count += 1;
                }
            }
            if unlinked_count > 0 {
                set_flag(ctx.core, owner, "l25_lianhuan_draw", unlinked_count);
                ctx.start_draw(owner, unlinked_count);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newsance(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) && m.stage == HostStage::Act {
        set_flag(ctx.core, owner, "l25_newsance_hand_plus2", 0);
        set_flag(ctx.core, owner, "mark:ssr_newsance_adds", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWSANCE_MODE),
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(opt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newsance");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newsance");
            set_flag(ctx.core, owner, "l25_newsance_used", cnt);
            set_flag(ctx.core, owner, "l25_newsance_hand_plus2", 1);
            set_flag(ctx.core, owner, "mark:ssr_newsance_adds", 1);
            set_flag(ctx.core, owner, "l25_newsance_mode", u32::from(opt) + 1);
            let unlock = match opt {
                0 => suit_bit(Suit::Spade),
                1 => suit_bit(Suit::Diamond),
                _ => suit_bit(Suit::Heart),
            };
            let mask = flag(ctx.core, owner, "l25_lianhuan_suit_mask") | unlock;
            set_flag(ctx.core, owner, "l25_lianhuan_suit_mask", mask);
            if opt == 0 || opt == 1 {
                if alive_others(ctx.core, owner).is_empty() {
                    set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                    return Flow::Done;
                }
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWSANCE_TARGET);
            }
            set_flag(ctx.core, owner, "l25_newsance_shoucheng", 1);
            let revealed = pop_top_deck_cards(ctx, 3);
            if revealed.is_empty() {
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            }
            m.cards = revealed;
            wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWSANCE_CARD)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let mode = flag(ctx.core, owner, "l25_newsance_mode").saturating_sub(1);
            if mode == 0 {
                set_flag(ctx.core, owner, "l25_newsance_xicheng", 1);
                let sha_c = ctx
                    .core
                    .player(tgt)
                    .hand
                    .iter()
                    .copied()
                    .find(|&c| is_sha_card(ctx.catalog, c));
                if let Some(sc) = sha_c {
                    discard_he_cards(ctx, tgt, &[sc]);
                    return Flow::Done;
                }
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    1,
                    DamageNature::Normal,
                    None,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            set_flag(ctx.core, owner, "l25_newsance_weicheng", 1);
            m.target = Some(tgt);
            ctx.start_draw(tgt, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(tgt) = m.target {
                if let Some(&cid) = ctx.core.player(tgt).hand.first() {
                    ctx.give_hand_card(tgt, owner, cid);
                }
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let chosen = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if m.cards.contains(&c) => c,
                _ => {
                    let Some(&first) = m.cards.first() else {
                        return Flow::Done;
                    };
                    first
                }
            };
            ctx.core.player_mut(owner).hand.push(chosen);
            ctx.out.push(LogEvent::CardsDrawn {
                player: owner,
                cards: vec![chosen],
            });
            for &c in &m.cards {
                if c != chosen {
                    ctx.core.deck.push(c);
                }
            }
            if card_suit(ctx.catalog, chosen) == Suit::Heart
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            {
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newxiebian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWXIEBIAN_MODE),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_xiebian_stance", u32::from(mode) + 1);
            set_flag(ctx.core, owner, "mark:ssr_newxiebian", u32::from(mode) + 1);
            match mode {
                0 => {
                    set_flag(ctx.core, owner, "l25_xiebian_futu", 1);
                    let judges = std::mem::take(&mut ctx.core.player_mut(owner).judge);
                    if !judges.is_empty() {
                        ctx.core.player_mut(owner).hand.extend(judges);
                        return Flow::Done;
                    }
                    if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                        ctx.recover(owner, 1, None);
                    }
                    ctx.start_draw(owner, 2);
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
                1 => {
                    set_flag(ctx.core, owner, "l25_xiebian_fantian", 1);
                    let mut gained = 0u32;
                    for p in alive_others(ctx.core, owner) {
                        if gained >= 2 {
                            break;
                        }
                        if let Some(&cid) = ctx.core.player(p).hand.first() {
                            ctx.give_hand_card(p, owner, cid);
                            gained += 1;
                        }
                    }
                    if gained == 0 {
                        ctx.start_draw(owner, 2);
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                2 => {
                    set_flag(ctx.core, owner, "l25_xiebian_xiuluo", 1);
                    if let Some(tgt) = alive_others(ctx.core, owner).into_iter().next() {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(owner),
                            tgt,
                            1,
                            DamageNature::Normal,
                            None,
                        )));
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                _ => {
                    set_flag(ctx.core, owner, "l25_xiebian_shijia", 1);
                    let mut discarded = false;
                    for p in alive_others(ctx.core, owner) {
                        if let Some(&cid) = ctx.core.player(p).hand.first() {
                            discard_he_cards(ctx, p, &[cid]);
                            discarded = true;
                            break;
                        }
                    }
                    ctx.start_draw(owner, if discarded { 1 } else { 2 });
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
            }
        }
        _ => Flow::Done,
    }
}

fn step_ssr_motu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    if ctx.core.player(owner).hp <= 1
        && !ctx.core.player(owner).hand.is_empty()
        && used_this_round(ctx.core, owner, "ssr_motu") < 1
    {
        inc_used_this_round(ctx.core, owner, "ssr_motu");
        if let Some(&cid) = ctx.core.player(owner).hand.first() {
            discard_he_cards(ctx, owner, &[cid]);
        }
        let cur_hujia = ctx.core.player(owner).hujia;
        ctx.core.player_mut(owner).hujia = cur_hujia.saturating_add(1).min(5);
        set_flag(ctx.core, owner, "l25_motu_triggered", 1);
    }
    Flow::Done
}

fn step_ssr_xunxun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.core.player(owner).hujia > 1 {
                let excess = ctx.core.player(owner).hujia - 1;
                ctx.core.player_mut(owner).hujia = 1;
                set_flag(ctx.core, owner, "l25_xunxun_diaobing", 1);
                ctx.start_draw(owner, u32::from(excess));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            if ctx.core.player(owner).hujia > 0 {
                return Flow::Done;
            }
            let revealed = pop_top_deck_cards(ctx, 4);
            if revealed.is_empty() {
                return Flow::Done;
            }
            if revealed
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_equip())
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            {
                ctx.recover(owner, 1, None);
                set_flag(ctx.core, owner, "l25_xunxun_recovered", 1);
            }
            m.cards = revealed;
            wait_ask(m, ctx, HostStage::SelectCard, ASK_XUNXUN_CARDS)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            for &c in &m.cards {
                if !chosen.contains(&c) {
                    ctx.core.deck.insert(0, c);
                }
            }
            for &c in &chosen {
                ctx.core.deck.push(c);
            }
            set_flag(ctx.core, owner, "l25_xunxun_rearranged", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_jieliang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(104) | Some(115)) && m.stage == HostStage::Act {
        let Some(p) = m.target else {
            return Flow::Done;
        };
        if flag(ctx.core, p, "l25_jieliang_jie") > 0 {
            let jcard = judge_one_card(ctx);
            let is_club = jcard
                .map(|c| card_suit(ctx.catalog, c) == Suit::Club)
                .unwrap_or(false);
            if !is_club {
                if let Some(cid) = ctx.core.player_mut(p).hand.pop() {
                    let mut pile = jieliang_pile(ctx.core, p);
                    pile.push(cid);
                    let cnt = pile.len() as u32;
                    ctx.core
                        .vm_aux
                        .piles
                        .insert((p, "ssr_jieliang".to_string()), pile);
                    set_flag(ctx.core, p, "l25_jieliang_liang_count", cnt);
                    set_flag(ctx.core, p, "mark:ssr_jieliang", cnt);
                }
            }
        }
        return Flow::Done;
    }
    if m.card == Some(101) && m.stage == HostStage::Act {
        let Some(tgt) = m.target else {
            return Flow::Done;
        };
        let Some(&src) = m.targets.first() else {
            return Flow::Done;
        };
        let mut pile = jieliang_pile(ctx.core, tgt);
        if !pile.is_empty() {
            let cid = pile.remove(0);
            let rem = pile.len() as u32;
            if pile.is_empty() {
                ctx.core
                    .vm_aux
                    .piles
                    .remove(&(tgt, "ssr_jieliang".to_string()));
            } else {
                ctx.core
                    .vm_aux
                    .piles
                    .insert((tgt, "ssr_jieliang".to_string()), pile);
            }
            set_flag(ctx.core, tgt, "l25_jieliang_liang_count", rem);
            set_flag(ctx.core, tgt, "mark:ssr_jieliang", rem);
            ctx.core.player_mut(src).hand.push(cid);
            ctx.out.push(LogEvent::CardsDrawn {
                player: src,
                cards: vec![cid],
            });
            set_flag(ctx.core, owner, "l25_jieliang_looted", 1);
        }
        return Flow::Done;
    }
    if m.card == Some(111) && m.stage == HostStage::Act {
        if let Some(p) = m.target {
            let jie = flag(ctx.core, p, "l25_jieliang_jie");
            if jie > 0 {
                set_flag(ctx.core, p, "l25_jieliang_jie", jie - 1);
                set_flag(ctx.core, p, "mark:ssr_jieliang_break", jie - 1);
            } else if let Some(cards) = ctx
                .core
                .vm_aux
                .piles
                .remove(&(p, "ssr_jieliang".to_string()))
            {
                if !cards.is_empty() {
                    ctx.core.player_mut(p).hand.extend(cards);
                    set_flag(ctx.core, p, "l25_jieliang_liang_count", 0);
                    set_flag(ctx.core, p, "mark:ssr_jieliang", 0);
                }
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_JIELIANG_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_jieliang");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_JIELIANG_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_jieliang");
                return Flow::Done;
            };
            let disc = std::mem::take(&mut m.cards);
            discard_he_cards(ctx, owner, &disc);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jieliang");
            set_flag(ctx.core, owner, "l25_jieliang_used", cnt);
            let next = flag(ctx.core, tgt, "l25_jieliang_jie") + 1;
            set_flag(ctx.core, tgt, "l25_jieliang_jie", next);
            set_flag(ctx.core, tgt, "mark:ssr_jieliang_break", next);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

