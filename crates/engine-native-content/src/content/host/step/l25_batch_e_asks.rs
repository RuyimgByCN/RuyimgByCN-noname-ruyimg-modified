//! L25 祖安武将 Batch E 询问构造与前半技能步进（技能 1–12：`ssr_yushui` 至 `ssr_newluanwu`）。

use super::l25_batch_e_ext::handle_l25_batch_e_ext;
use super::*;
use crate::catalog::{CardKind, Catalog, Faction, Suit};
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
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_JIEYIN_GIVE => card_ask(
            owner,
            "ssr_jieyin:give_card",
            red_hand_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_JIEYIN_TARGET => player_ask(
            owner,
            "ssr_jieyin:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_JIEYIN_GAIN => card_ask(
            owner,
            "ssr_jieyin:choose_card",
            hej_cards(core, tgt),
            1,
            1,
            false,
        ),
        ASK_WEIGONG_DISCARD => card_ask(
            owner,
            "ssr_weigong:discard_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_WEIGONG_GAIN => card_ask(
            owner,
            "ssr_weigong:choose_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_TIANLANG_CONFIRM => confirm_ask(owner, "ssr_tianlang:confirm"),
        ASK_TIANLANG_STAR => tag_ask(owner, "ssr_tianlang:choose_option", 0..=6, false),
        ASK_NEWJILUE_CONFIRM => confirm_ask(owner, "ssr_newjilue:confirm"),
        ASK_NEWJIJIE_CARD => card_ask(
            owner,
            "ssr_newjijie:choose_card",
            newjijie_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_NEWJIJIE_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !hej_cards(core, p).is_empty())
                .collect();
            player_ask(owner, "ssr_newjijie:choose_target", cands, 1, 1, false)
        }
        ASK_NEWJIYUAN_CONFIRM => confirm_ask(owner, "ssr_newjiyuan:confirm"),
        ASK_NEWJIYUAN_TARGET => player_ask(
            owner,
            "ssr_newjiyuan:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_JIQIAO_HAND => card_ask(
            owner,
            "ssr_jiqiao:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_JIQIAO_PILE => {
            let len = jiqiao_pile(core, owner).len().min(8) as u8;
            if len == 0 {
                return None;
            }
            tag_ask(owner, "ssr_jiqiao:choose_option", 0..len, false)
        }
        ASK_QICAI_TARGET => player_ask(
            owner,
            "ssr_qicai:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_NEWWEIMU_CARD => card_ask(
            owner,
            "ssr_newweimu:choose_card",
            black_he_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_NEWWEIMU_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !hej_cards(core, p).is_empty())
                .collect();
            player_ask(owner, "ssr_newweimu:choose_target", cands, 1, 1, false)
        }
        ASK_GUIMOU_CONFIRM => confirm_ask(owner, "ssr_guimou:confirm"),
        ASK_GUIMOU_TARGETS => {
            let cands: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_guimou:choose_targets", cands, 2, 2, true)
        }
        ASK_GUIMOU_PINDIAN_1 => card_ask(
            tgt,
            "ssr_guimou:pindian_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_GUIMOU_PINDIAN_2 => card_ask(
            tgt,
            "ssr_guimou:target_pindian_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_NEWLUANWU_CONFIRM => confirm_ask(owner, "ssr_newluanwu:confirm"),
        ASK_NEWLUANWU_MODE => tag_ask(tgt, "ssr_newluanwu:choose_mode", 0..=1, false),
        ASK_NIANLEI_GIVE => card_ask(
            owner,
            "ssr_nianlei:give_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_NIANLEI_TARGET => player_ask(
            owner,
            "ssr_nianlei:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_NEWDUANXIAN_DISCARD => card_ask(
            owner,
            "ssr_newduanxian:discard_cards",
            he_cards(core, owner),
            2,
            2,
            true,
        ),
        ASK_ZAIQI_DISCARD => card_ask(
            owner,
            "ssr_zaiqi:discard_cards",
            he_cards(core, owner),
            2,
            2,
            true,
        ),
        ASK_ZAIQI_CONFIRM => confirm_ask(owner, "ssr_zaiqi:confirm"),
        ASK_LIEREN_CARD => card_ask(
            owner,
            "ssr_lieren:choose_card",
            lieren_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_LIEREN_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| lieren_pile(core, p).is_empty())
                .collect();
            player_ask(owner, "ssr_lieren:choose_target", cands, 1, 1, false)
        }
        ASK_LIEREN_RESPONSE => {
            let shans: Vec<CardId> = core
                .player(tgt)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c) == CardKind::Shan)
                .collect();
            card_ask(tgt, "ssr_lieren:target_response", shans, 1, 1, true)
        }
        ASK_ZHIJI_OPTION => {
            let mut opts = Vec::new();
            if flag(core, owner, "l25_zhiji_zhengnan") == 0 {
                opts.push(0);
            }
            if flag(core, owner, "l25_zhiji_fabei") == 0 {
                opts.push(1);
            }
            if flag(core, owner, "l25_zhiji_wanlan") == 0 {
                opts.push(2);
            }
            if opts.is_empty() {
                opts = vec![0, 1, 2];
            }
            tag_ask(owner, "ssr_zhiji:choose_option", opts, false)
        }
        ASK_WEILONG_CARD => card_ask(
            owner,
            "ssr_weilong:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_WEILONG_TARGET => player_ask(
            owner,
            "ssr_weilong:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_HUNZI_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_hunzi:choose_target", cands, 1, 1, true)
        }
        ASK_HUNZI_PINDIAN => card_ask(
            owner,
            "ssr_hunzi:pindian_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_HUNZI_TARGET_PINDIAN => card_ask(
            tgt,
            "ssr_hunzi:target_pindian_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_NEWZHIBA_DISCARD => {
            let resp = alive_others(core, owner)
                .into_iter()
                .find(|&p| {
                    core.effective_faction(catalog, p) == Some(Faction::Wu)
                        && !he_cards(core, p).is_empty()
                })
                .or_else(|| {
                    alive_others(core, owner)
                        .into_iter()
                        .find(|&p| !he_cards(core, p).is_empty())
                })
                .unwrap_or(tgt);
            card_ask(
                resp,
                "ssr_newzhiba:discard_card",
                he_cards(core, resp),
                1,
                1,
                true,
            )
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
    set_flag(ctx.core, m.owner, "l25_batch_e_ask_id", ask_id);
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
        set_flag(ctx.core, m.owner, "l25_batch_e_ask_id", 0);
        Flow::Done
    }
}

pub fn ask_l25_batch_e(
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
        .unwrap_or_else(|| flag(core, owner, "l25_batch_e_ask_id"));
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

pub(super) fn handle_l25_batch_e_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let cur_ask = flag(ctx.core, owner, "l25_batch_e_ask_id");
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::CardsAsk
        && cur_ask != ASK_LIEREN_RESPONSE
        && ctx.res.answer.as_ref().is_some_and(|a| a.is_empty())
    {
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ssr_yushui" => step_ssr_yushui(m, ctx),
        "ssr_jieyin" => step_ssr_jieyin(m, ctx),
        "ssr_weigong" => step_ssr_weigong(m, ctx),
        "ssr_tianlang" => step_ssr_tianlang(m, ctx),
        "ssr_newjilue" => step_ssr_newjilue(m, ctx),
        "ssr_newjijie" => step_ssr_newjijie(m, ctx),
        "ssr_newjiyuan" => step_ssr_newjiyuan(m, ctx),
        _ => handle_l25_batch_e_ext(m, ctx),
    }
}

fn step_ssr_yushui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    let mut prevented_in_place = false;
    if ctx.current_damage().is_ok() {
        let d = ctx.current_damage_mut();
        if !d.prevented && d.amount > 0 {
            d.prevented = true;
            d.amount = 0;
            prevented_in_place = true;
        }
    }
    if !prevented_in_place {
        ctx.recover(owner, 1, None);
    }
    set_flag(ctx.core, owner, "l25_yushui_triggered", 1);
    let my_he = he_cards(ctx.core, owner);
    if !my_he.is_empty() {
        ctx.out.push(LogEvent::CardsRevealed {
            cards: my_he.clone(),
            to: Vec::new(),
        });
    }
    let existing_suits: Vec<Suit> = my_he
        .into_iter()
        .map(|c| ctx.catalog.effective_card_suit(ctx.core, owner, c))
        .collect();
    if let Some(cid) =
        fetch_card_matching(ctx, |c, cat| !existing_suits.contains(&card_suit(cat, c)))
    {
        ctx.core.player_mut(owner).hand.push(cid);
        ctx.out.push(LogEvent::CardsDrawn {
            player: owner,
            cards: vec![cid],
        });
        set_flag(ctx.core, owner, "l25_yushui_gained", cid + 1);
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: 1,
        });
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_ssr_jieyin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        let mut drew = false;
        for p in alive_all(ctx.core, owner) {
            if flag(ctx.core, p, "l25_jieyin_hongzhuang") > 0
                && (m.target == Some(p) || m.target.is_none())
            {
                set_flag(ctx.core, p, "l25_jieyin_hongzhuang", 0);
                set_flag(ctx.core, p, "mark:ssr_jieyin_zhuang", 0);
                ctx.start_draw(p, 1);
                drew = true;
            }
        }
        if drew {
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_JIEYIN_GIVE),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_JIEYIN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jieyin");
            set_flag(ctx.core, owner, "l25_jieyin_used", cnt);
            if let Some(&cid) = m.cards.first() {
                if ctx.core.player(owner).hand.contains(&cid) {
                    ctx.give_hand_card(owner, tgt, cid);
                }
            }
            if hej_cards(ctx.core, tgt).is_empty() {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_JIEYIN_GAIN)
        }
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            if let Some(ChoiceOption::Card(gain_cid)) = ctx.answer().first().copied() {
                gain_from_player(ctx, owner, tgt, gain_cid);
            }
            if flag(ctx.core, tgt, "l25_jieyin_hongzhuang") == 0 {
                ctx.recover(owner, 1, None);
                set_flag(ctx.core, tgt, "l25_jieyin_hongzhuang", 1);
                set_flag(ctx.core, tgt, "mark:ssr_jieyin_zhuang", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_weigong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let atk = m.target.unwrap_or(owner);
    match m.stage {
        HostStage::Act => {
            if ctx.core.player(atk).hand.is_empty() {
                return Flow::Done;
            }
            if flag(ctx.core, atk, "l25_jieyin_hongzhuang") == 0 {
                if ctx.core.player(atk).hand.len() == 1 {
                    let cid = ctx.core.player(atk).hand[0];
                    ctx.discard_from_hand(atk, cid);
                    set_flag(ctx.core, owner, "l25_weigong_triggered", 1);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_WEIGONG_DISCARD)
            } else {
                if ctx.core.player(atk).hand.len() == 1 {
                    let cid = ctx.core.player(atk).hand[0];
                    if atk != owner {
                        ctx.give_hand_card(atk, owner, cid);
                    }
                    set_flag(ctx.core, owner, "l25_weigong_triggered", 2);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_WEIGONG_GAIN)
            }
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                if ctx.core.player(atk).hand.contains(&cid) {
                    ctx.discard_from_hand(atk, cid);
                }
                set_flag(ctx.core, owner, "l25_weigong_triggered", 1);
            }
            Flow::Done
        }
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                if atk != owner && ctx.core.player(atk).hand.contains(&cid) {
                    ctx.give_hand_card(atk, owner, cid);
                }
                set_flag(ctx.core, owner, "l25_weigong_triggered", 2);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_tianlang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_TIANLANG_CONFIRM),
        HostStage::SelectNature => {
            if !is_confirmed(ctx) {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::JiefenTarget, ASK_TIANLANG_STAR)
        }
        HostStage::JiefenTarget => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let tag = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v.min(6),
                _ => 0,
            };
            let star = u32::from(tag) + 1;
            set_flag(ctx.core, owner, "l25_tianlang_star", star);
            set_flag(ctx.core, owner, "l25_tianlang_triggered", 1);
            match star {
                1 => {
                    ctx.recover(owner, 1, None);
                    ctx.start_draw(owner, 1);
                }
                2 => {
                    ctx.start_draw(owner, 2);
                }
                3 => {
                    let hj = (ctx.core.player(owner).hujia + 1).min(5);
                    ctx.core.player_mut(owner).hujia = hj;
                    ctx.start_draw(owner, 1);
                }
                4 => {
                    if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                        let jc = ctx.draw_judge_card();
                        ctx.out.push(LogEvent::Judged {
                            player: owner,
                            card: jc,
                        });
                        ctx.core.player_mut(owner).hand.push(jc);
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: owner,
                            cards: vec![jc],
                        });
                    }
                    ctx.start_draw(owner, 1);
                }
                5 => {
                    if let Some(other) = alive_others(ctx.core, owner).first().copied() {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(owner),
                            other,
                            1,
                            DamageNature::Normal,
                            None,
                        )));
                    }
                    ctx.start_draw(owner, 1);
                }
                6 => {
                    if let Some(other) = alive_others(ctx.core, owner)
                        .into_iter()
                        .find(|&p| !he_cards(ctx.core, p).is_empty())
                    {
                        if let Some(&cid) = he_cards(ctx.core, other).first() {
                            discard_he_cards(ctx, other, &[cid]);
                        }
                    }
                    ctx.start_draw(owner, 1);
                }
                _ => {
                    ctx.recover(owner, 1, None);
                    ctx.start_draw(owner, 2);
                }
            }
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newjilue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWJILUE_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l25_newjilue_reflected", 1);
            let atk = m.target.unwrap_or(owner);
            let kind = m
                .cards
                .first()
                .map(|&c| ctx.catalog.card_kind(c))
                .filter(|k| !k.is_basic_kind() && !k.is_equip())
                .unwrap_or(CardKind::Dismantle);
            if atk != owner && ctx.core.player(atk).alive {
                if !ctx.core.deck.is_empty() {
                    ctx.use_virtual_card(owner, kind, vec![atk]);
                } else {
                    ctx.start_draw(owner, 1);
                }
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newjijie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(110) {
        set_flag(ctx.core, owner, "l25_newjijie_type_mask", 0);
        return Flow::Done;
    }
    if m.card == Some(102) || m.card == Some(103) {
        if let Some(&cid) = m.cards.first() {
            let ts = turn_stamp(ctx.core);
            let prev = if flag(ctx.core, owner, "l25_newjijie_mask_stamp") == ts {
                flag(ctx.core, owner, "l25_newjijie_type_mask")
            } else {
                0
            };
            set_flag(ctx.core, owner, "l25_newjijie_mask_stamp", ts);
            set_flag(
                ctx.core,
                owner,
                "l25_newjijie_type_mask",
                prev | card_type_bit(ctx.catalog, cid),
            );
            set_flag(ctx.core, owner, "l25_newjijie_used", 1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWJIJIE_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWJIJIE_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newjijie");
            set_flag(ctx.core, owner, "l25_newjijie_used", cnt);
            if let Some(&cid) = m.cards.first() {
                let ts = turn_stamp(ctx.core);
                let prev = if flag(ctx.core, owner, "l25_newjijie_mask_stamp") == ts {
                    flag(ctx.core, owner, "l25_newjijie_type_mask")
                } else {
                    0
                };
                set_flag(ctx.core, owner, "l25_newjijie_mask_stamp", ts);
                set_flag(
                    ctx.core,
                    owner,
                    "l25_newjijie_type_mask",
                    prev | card_type_bit(ctx.catalog, cid),
                );
                discard_he_cards(ctx, owner, &[cid]);
            }
            if let Some(&victim_cid) = hej_cards(ctx.core, tgt).first() {
                discard_from_hej(ctx, tgt, victim_cid);
            }
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn give_all_handcards(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId) {
    if from == to || !ctx.core.player(to).alive {
        return;
    }
    let hand = ctx.core.player(from).hand.clone();
    for cid in hand {
        ctx.give_hand_card(from, to, cid);
    }
}

fn step_ssr_newjiyuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        let mut present_mask = 0u32;
        for &cid in &ctx.core.player(owner).hand {
            present_mask |= card_type_bit(ctx.catalog, cid);
        }
        let missing_mask = (!present_mask) & 0b111;
        if missing_mask != 0 {
            if let Some(cid) =
                fetch_card_matching(ctx, |c, cat| (card_type_bit(cat, c) & missing_mask) != 0)
            {
                ctx.core.player_mut(owner).hand.push(cid);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![cid],
                });
                set_flag(ctx.core, owner, "l25_newjiyuan_fetched", 1);
            } else {
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "l25_newjiyuan_fetched", 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWJIYUAN_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            if let Some(tgt) = m.target.filter(|&p| p != owner && ctx.core.player(p).alive) {
                give_all_handcards(ctx, owner, tgt);
                set_flag(ctx.core, owner, "l25_newjiyuan_gave", 1);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWJIYUAN_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                give_all_handcards(ctx, owner, tgt);
                set_flag(ctx.core, owner, "l25_newjiyuan_gave", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

