//! L25 祖安武将 Batch F 询问构造与前半技能步进（技能 1–8：`ssr_kuiwei` 至 `ssr_zhuzhen`）。

use super::l25_batch_f_ext::handle_l25_batch_f_ext;
use super::*;
use crate::catalog::{CardKind, Catalog};
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

pub(super) fn gain_from_player(
    ctx: &mut Ctx<'_>,
    to: PlayerId,
    from: PlayerId,
    cid: CardId,
) {
    if ctx.core.player(from).hand.contains(&cid) {
        if from != to {
            ctx.give_hand_card(from, to, cid);
        }
    } else if ctx.core.player(from).equips.contains(&cid) {
        ctx.take_owned_card(from, cid);
        ctx.core.player_mut(to).hand.push(cid);
        ctx.emit(EventData::CardsMoved {
            cards: vec![cid],
            from,
            to,
        });
        ctx.out.push(LogEvent::CardsGiven {
            from,
            to,
            cards: vec![cid],
        });
    } else if let Some(pos) = ctx.core.player(from).judge.iter().position(|&x| x == cid) {
        ctx.core.player_mut(from).judge.remove(pos);
        ctx.core.player_mut(to).hand.push(cid);
        ctx.emit(EventData::CardsMoved {
            cards: vec![cid],
            from,
            to,
        });
    }
}

pub(super) fn discard_from_hej(ctx: &mut Ctx<'_>, p: PlayerId, cid: CardId) {
    if ctx.core.player(p).hand.contains(&cid) || ctx.core.player(p).equips.contains(&cid) {
        discard_he_cards(ctx, p, &[cid]);
    } else if let Some(pos) = ctx.core.player(p).judge.iter().position(|&x| x == cid) {
        ctx.core.player_mut(p).judge.remove(pos);
        ctx.to_discard(cid);
        ctx.out.push(LogEvent::CardDiscarded {
            player: p,
            cards: vec![cid],
        });
    }
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
        ASK_KUIWEI_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_kuiwei:choose_target", cands, 1, 1, true)
        }
        ASK_KUIWEI_PINDIAN => card_ask(
            owner,
            "ssr_kuiwei:pindian_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_KUIWEI_TARGET_PINDIAN => card_ask(
            tgt,
            "ssr_kuiwei:target_pindian_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_HUWEI_CONFIRM => confirm_ask(owner, "ssr_huwei:confirm"),
        ASK_SHENSU_MODE => {
            let range = if shensu_marks(core, owner) >= 2 {
                0..=1
            } else {
                0..=0
            };
            tag_ask(owner, "ssr_shensu:choose_mode", range, true)
        }
        ASK_HUBU_TARGET => player_ask(
            owner,
            "ssr_hubu:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_HUBU_OPTION => {
            let max_n = (shensu_marks(core, owner) / 2).clamp(1, 3) as u8;
            tag_ask(owner, "ssr_hubu:choose_option", 0..=max_n, false)
        }
        ASK_JUSHOU_CARD => card_ask(
            owner,
            "ssr_jushou:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_JUSHOU_OPTION => {
            let len = jushou_pile(core, owner).len().max(1).min(8) as u8;
            tag_ask(owner, "ssr_jushou:choose_option", 0..len, false)
        }
        ASK_ZHUZHEN_CARD => card_ask(
            owner,
            "ssr_zhuzhen:choose_card",
            jushou_pile(core, owner),
            1,
            1,
            true,
        ),
        ASK_ZHUZHEN_TARGET => player_ask(
            owner,
            "ssr_zhuzhen:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_TUNTIAN_DISCARD => card_ask(
            owner,
            "ssr_tuntian:discard_cards",
            tuntian_eligible_cards(core, catalog, owner),
            2,
            2,
            true,
        ),
        ASK_TUNTIAN_CHOOSE_CARD => {
            let pool: Vec<CardId> = if !cards.is_empty() {
                cards.to_vec()
            } else {
                let n = tuntian_marks(core, owner).clamp(1, 4) as usize;
                core.deck.iter().rev().take(n).copied().collect()
            };
            card_ask(owner, "ssr_tuntian:choose_card", pool, 1, 1, false)
        }
        ASK_NEWHENGZHENG_MODE => tag_ask(owner, "ssr_newhengzheng:choose_mode", 0..=2, false),
        ASK_NEWJIUCHI_CONFIRM => confirm_ask(owner, "ssr_newjiuchi:confirm"),
        ASK_NEWMIZHOU_CARD => card_ask(
            owner,
            "ssr_newmizhou:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_NEWMIZHOU_OPTION => tag_ask(owner, "ssr_newmizhou:choose_option", 0..=3, false),
        ASK_NEWLUANJI_DISCARD => card_ask(
            owner,
            "ssr_newluanji:discard_cards",
            he_cards(core, owner),
            2,
            2,
            true,
        ),
        ASK_NEWSHIYI_CONFIRM => confirm_ask(owner, "ssr_newshiyi:confirm"),
        ASK_ZHUHAI_DISCARD => card_ask(
            owner,
            "ssr_zhuhai:discard_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_ZHUHAI_TARGET => {
            let mut cands = Vec::new();
            if let Some(t) = target {
                if t != owner && core.player(t).alive {
                    cands.push(t);
                }
            }
            for p in alive_others(core, owner) {
                if !cands.contains(&p) {
                    cands.push(p);
                }
            }
            player_ask(owner, "ssr_zhuhai:choose_target", cands, 1, 1, false)
        }
        ASK_YIJIAN_TARGET => player_ask(
            owner,
            "ssr_yijian:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_NEWFABING_OPTION => tag_ask(owner, "ssr_newfabing:choose_option", 0..=3, true),
        ASK_XUANHUO_DISCARD => card_ask(
            owner,
            "ssr_xuanhuo:discard_card",
            xuanhuo_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_XUANHUO_TARGET => player_ask(
            owner,
            "ssr_xuanhuo:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
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
    set_flag(ctx.core, m.owner, "l25_batch_f_ask_id", ask_id);
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
        set_flag(ctx.core, m.owner, "l25_batch_f_ask_id", 0);
        Flow::Done
    }
}

pub fn ask_l25_batch_f(
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
        .unwrap_or_else(|| flag(core, owner, "l25_batch_f_ask_id"));
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

pub(super) fn handle_l25_batch_f_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::CardsAsk
        && ctx.res.answer.as_ref().is_some_and(|a| a.is_empty())
    {
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ssr_kuiwei" => step_ssr_kuiwei(m, ctx),
        "ssr_newhanzhan" => step_ssr_newhanzhan(m, ctx),
        "ssr_newbuqu" => step_ssr_newbuqu(m, ctx),
        "ssr_huwei" => step_ssr_huwei(m, ctx),
        "ssr_shensu" => step_ssr_shensu(m, ctx),
        "ssr_hubu" => step_ssr_hubu(m, ctx),
        "ssr_jushou" => step_ssr_jushou(m, ctx),
        "ssr_zhuzhen" => step_ssr_zhuzhen(m, ctx),
        _ => handle_l25_batch_f_ext(m, ctx),
    }
}

fn step_ssr_kuiwei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        for p in 0..ctx.core.players.len() as PlayerId {
            set_flag(ctx.core, p, "l25_kuiwei_forbid_basic", 0);
            set_flag(ctx.core, p, "mark:ssr_kuiwei_break", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_KUIWEI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_kuiwei");
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_KUIWEI_PINDIAN)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_kuiwei");
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![c1];
            wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_KUIWEI_TARGET_PINDIAN)
        }
        HostStage::CompareTargetPick => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let Some(&c1) = m.cards.first() else {
                return Flow::Done;
            };
            let c2 = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(cid)) => cid,
                _ => {
                    let Some(&fallback) = ctx.core.player(tgt).hand.first() else {
                        return Flow::Done;
                    };
                    fallback
                }
            };
            let owner_used_sha = is_sha_card(ctx.catalog, c1);
            let mut r1 = card_rank(ctx.catalog, c1);
            let mut r2 = card_rank(ctx.catalog, c2);
            let owner_lost =
                (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u8;
            if owner_used_sha && owner_lost > 0 {
                r1 = (r1.saturating_add(owner_lost)).min(13);
                set_flag(ctx.core, owner, "l25_newhanzhan_boost", u32::from(owner_lost));
            }
            let tgt_has_hanzhan = ctx
                .catalog
                .skill_by_name("ssr_newhanzhan")
                .is_some_and(|def| ctx.core.skills.contains_key(&(tgt, def)));
            let tgt_lost =
                (ctx.core.player(tgt).max_hp as i32 - ctx.core.player(tgt).hp).max(0) as u8;
            if tgt_has_hanzhan && is_sha_card(ctx.catalog, c2) && tgt_lost > 0 {
                r2 = (r2.saturating_add(tgt_lost)).min(13);
                set_flag(ctx.core, tgt, "l25_newhanzhan_boost", u32::from(tgt_lost));
            }
            if ctx.core.player(owner).hand.contains(&c1) {
                ctx.discard_from_hand(owner, c1);
            }
            if ctx.core.player(tgt).hand.contains(&c2) {
                ctx.discard_from_hand(tgt, c2);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_kuiwei");
            set_flag(ctx.core, owner, "l25_kuiwei_used", cnt);
            if owner_used_sha {
                if r1 != r2 {
                    let (winner, loser) = if r1 > r2 { (owner, tgt) } else { (tgt, owner) };
                    set_flag(ctx.core, winner, "l25_kuiwei_win", 1);
                    set_flag(ctx.core, loser, "l25_kuiwei_forbid_basic", 1);
                    set_flag(ctx.core, loser, "mark:ssr_kuiwei_break", 1);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(winner),
                        loser,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                    m.stage = HostStage::CardsAsk;
                    return Flow::Yield;
                }
            } else {
                let higher = if r1 >= r2 { c1 } else { c2 };
                if let Some(pos) = ctx.core.discard.iter().rposition(|&x| x == higher) {
                    ctx.core.discard.remove(pos);
                }
                ctx.core.player_mut(owner).hand.push(higher);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![higher],
                });
                set_flag(ctx.core, owner, "l25_kuiwei_gained", higher + 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newhanzhan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    let lost_hp = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
    if lost_hp > 0 {
        set_flag(ctx.core, owner, "l25_newhanzhan_boost", lost_hp);
    }
    set_flag(ctx.core, owner, "l25_newhanzhan_active", 1);
    Flow::Done
}

fn step_ssr_newbuqu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    if m.card == Some(102) {
        set_flag(ctx.core, owner, "l25_buqu_sha_directhit", 1);
        return Flow::Done;
    }
    if flag(ctx.core, owner, "l25_buqu_first_triggered") == 0 {
        set_flag(ctx.core, owner, "l25_buqu_first_triggered", 1);
        let hp = ctx.core.player(owner).hp;
        if hp < 1 {
            ctx.recover(owner, (1 - hp) as u32, None);
        }
        return Flow::Done;
    }
    let Some(c) = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop()) else {
        let hp = ctx.core.player(owner).hp;
        if hp < 1 {
            ctx.recover(owner, (1 - hp) as u32, None);
        }
        return Flow::Done;
    };
    ctx.out.push(LogEvent::CardsRevealed {
        cards: vec![c],
        to: Vec::new(),
    });
    let r = card_rank(ctx.catalog, c);
    let mut pile = newbuqu_pile(ctx.core, owner);
    let dup = pile.iter().any(|&ex| card_rank(ctx.catalog, ex) == r);
    if !dup {
        pile.push(c);
        let len = pile.len() as u32;
        ctx.core
            .vm_aux
            .piles
            .insert((owner, "ssr_newbuqu".to_string()), pile);
        set_flag(ctx.core, owner, "l25_buqu_chuang_count", len);
        set_flag(ctx.core, owner, "mark:ssr_newbuqu", len);
        let hp = ctx.core.player(owner).hp;
        if hp < 1 {
            ctx.recover(owner, (1 - hp) as u32, None);
        }
    } else {
        ctx.to_discard(c);
        ctx.out.push(LogEvent::CardDiscarded {
            player: owner,
            cards: vec![c],
        });
        set_flag(ctx.core, owner, "l25_buqu_duplicate_discarded", 1);
    }
    Flow::Done
}

fn step_ssr_huwei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_HUWEI_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let cnt = inc_used_this_round(ctx.core, owner, "ssr_huwei");
            set_flag(ctx.core, owner, "l25_huwei_used", cnt);
            set_flag(ctx.core, owner, "l25_huwei_redirected", 1);
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_shensu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(101) {
        let amt = ctx
            .current_damage()
            .map(|d| d.amount.max(1) as u32)
            .unwrap_or(1);
        let next = shensu_marks(ctx.core, owner).saturating_add(3 * amt);
        set_shensu_marks(ctx.core, owner, next);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_SHENSU_MODE),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let cur = shensu_marks(ctx.core, owner);
            if mode == 1 && cur >= 2 {
                set_shensu_marks(ctx.core, owner, cur - 2);
                set_flag(ctx.core, owner, "l25_shensu_cleared", 1);
                let judges: Vec<CardId> = ctx.core.player(owner).judge.clone();
                for j in judges {
                    discard_from_hej(ctx, owner, j);
                }
            } else {
                set_shensu_marks(ctx.core, owner, cur + 3);
                set_flag(ctx.core, owner, "l25_shensu_skipped", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_hubu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_HUBU_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_HUBU_OPTION)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let max_n = (shensu_marks(ctx.core, owner) / 2).clamp(1, 3);
            let n = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(0)) | Some(ChoiceOption::Tag(1)) => 1,
                Some(ChoiceOption::Tag(v)) => u32::from(v).min(max_n).max(1),
                _ => 1,
            };
            let cur = shensu_marks(ctx.core, owner);
            set_shensu_marks(ctx.core, owner, cur.saturating_sub(2 * n));
            set_flag(ctx.core, tgt, "l25_hubu_unequip2", 1);
            set_flag(ctx.core, owner, "l25_hubu_sha_count", n);
            m.targets = vec![tgt; (n as usize).saturating_sub(1)];
            ctx.use_virtual_card(owner, CardKind::Sha, vec![tgt]);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(tgt) = m.targets.pop() {
                if ctx.core.player(owner).alive && ctx.core.player(tgt).alive {
                    ctx.use_virtual_card(owner, CardKind::Sha, vec![tgt]);
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn init_jushou_pile(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut pile = jushou_pile(ctx.core, owner);
    while pile.len() < 4 {
        if let Some(c) = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop()) {
            pile.push(c);
        } else {
            break;
        }
    }
    let len = pile.len() as u32;
    if !pile.is_empty() {
        ctx.core
            .vm_aux
            .piles
            .insert((owner, "ssr_jushou".to_string()), pile);
    }
    set_flag(ctx.core, owner, "l25_jushou_init", 1);
    set_flag(ctx.core, owner, "l25_jushou_cheng_count", len);
    set_flag(ctx.core, owner, "mark:ssr_jushou", len);
}

fn step_ssr_jushou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(108) | Some(110)) {
        init_jushou_pile(ctx, owner);
        return Flow::Done;
    }
    if m.card == Some(101) {
        if ctx.current_damage().is_ok() {
            let d = ctx.current_damage_mut();
            if d.amount > 1 {
                d.amount = 1;
            }
        } else {
            ctx.recover(owner, 1, None);
        }
        set_flag(ctx.core, owner, "l25_jushou_reduced", 1);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            if jushou_pile(ctx.core, owner).is_empty() && flag(ctx.core, owner, "l25_jushou_init") == 0 {
                init_jushou_pile(ctx, owner);
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_JUSHOU_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(h_cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_jushou");
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![h_cid];
            wait_ask(m, ctx, HostStage::SelectNature, ASK_JUSHOU_OPTION)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => usize::from(v),
                _ => 0,
            };
            let Some(&h_cid) = m.cards.first() else {
                return Flow::Done;
            };
            if ctx.core.player(owner).hand.contains(&h_cid) {
                discard_he_cards(ctx, owner, &[h_cid]);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jushou");
            set_flag(ctx.core, owner, "l25_jushou_used", cnt);
            let pile = jushou_pile(ctx.core, owner);
            let cheng_kind = pile
                .get(idx.min(pile.len().saturating_sub(1)))
                .map(|&c| ctx.catalog.card_kind(c))
                .unwrap_or(CardKind::Sha);
            match cheng_kind {
                CardKind::Tao => {
                    ctx.recover(owner, 1, None);
                    Flow::Done
                }
                CardKind::Jiu => {
                    ctx.core.turn.jiu_active = true;
                    Flow::Done
                }
                CardKind::SavageAttack | CardKind::ArcheryAttack => {
                    let targets = alive_others(ctx.core, owner);
                    if !targets.is_empty() {
                        ctx.use_virtual_card(owner, cheng_kind, targets);
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Duel => {
                    if let Some(tgt) = alive_others(ctx.core, owner).into_iter().next() {
                        ctx.use_virtual_card(owner, cheng_kind, vec![tgt]);
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                _ => {
                    ctx.start_draw(owner, 1);
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
            }
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zhuzhen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(101) {
        let helpers: Vec<PlayerId> = alive_others(ctx.core, owner)
            .into_iter()
            .filter(|&p| !jushou_pile(ctx.core, p).is_empty() || flag(ctx.core, p, "l25_zhuzhen_cheng") > 0)
            .collect();
        if helpers.is_empty() {
            return Flow::Done;
        }
        for &p in &helpers {
            ctx.start_draw(p, 1);
        }
        m.targets = helpers;
        m.stage = HostStage::DrawSettle;
        return Flow::Yield;
    }
    match m.stage {
        HostStage::DrawSettle => {
            for p in m.targets.clone() {
                if let Some(&cid) = ctx.core.player(p).hand.first() {
                    ctx.give_hand_card(p, owner, cid);
                }
            }
            set_flag(ctx.core, owner, "l25_zhuzhen_triggered", 1);
            Flow::Done
        }
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHUZHEN_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_zhuzhen");
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUZHEN_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_zhuzhen");
                return Flow::Done;
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            let mut my_pile = jushou_pile(ctx.core, owner);
            if let Some(pos) = my_pile.iter().position(|&x| x == cid) {
                my_pile.remove(pos);
            }
            let my_len = my_pile.len() as u32;
            if my_pile.is_empty() {
                ctx.core.vm_aux.piles.remove(&(owner, "ssr_jushou".to_string()));
            } else {
                ctx.core
                    .vm_aux
                    .piles
                    .insert((owner, "ssr_jushou".to_string()), my_pile);
            }
            set_flag(ctx.core, owner, "l25_jushou_cheng_count", my_len);
            let mut tgt_pile = jushou_pile(ctx.core, tgt);
            tgt_pile.push(cid);
            let tgt_len = tgt_pile.len() as u32;
            ctx.core
                .vm_aux
                .piles
                .insert((tgt, "ssr_jushou".to_string()), tgt_pile);
            set_flag(ctx.core, tgt, "l25_zhuzhen_cheng", tgt_len);
            set_flag(ctx.core, tgt, "mark:ssr_jushou", tgt_len);
            let total = flag(ctx.core, owner, "l25_zhuzhen_used_total") + 1;
            set_flag(ctx.core, owner, "l25_zhuzhen_used_total", total);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_zhuzhen");
            set_flag(ctx.core, owner, "l25_zhuzhen_used", cnt);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
