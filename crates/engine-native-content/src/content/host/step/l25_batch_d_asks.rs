//! L25 祖安武将 Batch D 询问构造与前半技能步进（技能 1–7：`ssr_fenchuan` 至 `ssr_yongsi`）。

use super::l25_batch_d_ext::handle_l25_batch_d_ext;
use super::*;
use crate::catalog::{Catalog, Faction};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
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
        ASK_FENCHUAN_SHOW => card_ask(
            tgt,
            "ssr_fenchuan:show_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_FENCHUAN_DISCARD => {
            let match_suit = cards
                .first()
                .map(|&c| catalog.effective_card_suit(core, tgt, c));
            let cands: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    match_suit
                        .map_or(true, |s| catalog.effective_card_suit(core, owner, c) == s)
                })
                .collect();
            card_ask(owner, "ssr_fenchuan:discard_card", cands, 1, 1, true)
        }
        ASK_NEWLIHUN_GIVE => card_ask(
            owner,
            "ssr_newlihun:give_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_NEWLIHUN_TARGET => player_ask(
            owner,
            "ssr_newlihun:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_NEWLIHUN_GUESS => tag_ask(tgt, "ssr_newlihun:target_choice", 0..=2, false),
        ASK_YJIXI_GAIN => card_ask(
            owner,
            "ssr_yjixi:choose_card",
            he_cards(core, tgt),
            1,
            1,
            false,
        ),
        ASK_YANJI_DISCARD => card_ask(
            owner,
            "ssr_yanji:discard_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_YANJI_TARGET => player_ask(
            owner,
            "ssr_yanji:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_CUANDI_CONFIRM => {
            let resp = alive_others(core, owner)
                .into_iter()
                .find(|&p| core.effective_faction(catalog, p) == Some(Faction::Qun))
                .or_else(|| alive_others(core, owner).into_iter().next())
                .unwrap_or(tgt);
            tag_ask(resp, "ssr_cuandi:target_choice", 0..=1, true)
        }
        ASK_CUANDI_DISCARD => card_ask(
            owner,
            "ssr_cuandi:discard_card",
            he_cards(core, owner),
            1,
            1,
            false,
        ),
        ASK_YONGSI_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "ssr_yongsi:choose_target", cands, 1, 1, true)
        }
        ASK_YONGSI_MODE => tag_ask(owner, "ssr_yongsi:choose_mode", 0..=1, false),
        ASK_YONGSI_GIVE => card_ask(
            owner,
            "ssr_yongsi:give_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_YONGSI_CONFIRM => confirm_ask(owner, "ssr_yongsi:confirm"),
        ASK_YONGSI_DISCARD => {
            let my_he = he_cards(core, owner);
            let max_d = my_he.len().min(2);
            card_ask(owner, "ssr_yongsi:discard_cards", my_he, 1, max_d, true)
        }
        ASK_YIJUE_CARD => card_ask(
            owner,
            "ssr_yijue:choose_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_YIJUE_TARGET => player_ask(
            owner,
            "ssr_yijue:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_YIJUE_CHOICE => tag_ask(tgt, "ssr_yijue:target_choice", 0..=1, false),
        ASK_NEWBAIYI_CARD => card_ask(
            owner,
            "ssr_newbaiyi:choose_card",
            baiyi_eligible_cards(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_NEWBAIYI_TARGET => {
            let cands: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| flag(core, p, "l25_baiyi_du") == 0)
                .collect();
            player_ask(owner, "ssr_newbaiyi:choose_target", cands, 1, 1, false)
        }
        ASK_LIECHENG_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .collect();
            player_ask(owner, "ssr_liecheng:choose_target", cands, 1, 1, true)
        }
        ASK_LIECHENG_CARD => card_ask(
            owner,
            "ssr_liecheng:choose_card",
            he_cards(core, tgt),
            1,
            1,
            false,
        ),
        ASK_NEWYUCONG_CARD => card_ask(
            owner,
            "ssr_newyucong:choose_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_NEWYUCONG_TARGET => {
            let cands: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| flag(core, p, "l25_yucong_mounted") == 0)
                .collect();
            player_ask(owner, "ssr_newyucong:choose_target", cands, 1, 1, false)
        }
        ASK_NEWGUOSE_MODE => tag_ask(owner, "ssr_newguose:choose_mode", 0..=1, true),
        ASK_NEWGUOSE_TARGET => player_ask(
            owner,
            "ssr_newguose:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_NEWGUOSE_PINDIAN => card_ask(
            owner,
            "ssr_newguose:pindian_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_NEWGUOSE_TARGET_PINDIAN => card_ask(
            tgt,
            "ssr_newguose:target_pindian_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_NEWTONGQUE_CONFIRM => confirm_ask(owner, "ssr_newtongque:confirm"),
        ASK_NEWTIANXIANG_PLAY_TARGET => player_ask(
            owner,
            "ssr_newtianxiang:choose_target",
            alive_all(core, owner),
            1,
            1,
            true,
        ),
        ASK_NEWTIANXIANG_DISCARD => card_ask(
            owner,
            "ssr_newtianxiang:discard_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_NEWTIANXIANG_DEFLECT_TARGET => player_ask(
            owner,
            "ssr_newtianxiang:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_NEWLUOYI_CONFIRM => confirm_ask(owner, "ssr_newluoyi:confirm"),
        ASK_ZHENHUN_CONFIRM => confirm_ask(owner, "ssr_zhenhun:confirm"),
        ASK_ZHENHUN_TARGET => player_ask(
            owner,
            "ssr_zhenhun:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_NEWLUOSHEN_CONFIRM => confirm_ask(owner, "ssr_newluoshen:confirm"),
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
    set_flag(ctx.core, m.owner, "l25_batch_d_ask_id", ask_id);
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
        set_flag(ctx.core, m.owner, "l25_batch_d_ask_id", 0);
        Flow::Done
    }
}

pub fn ask_l25_batch_d(
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
        .unwrap_or_else(|| flag(core, owner, "l25_batch_d_ask_id"));
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

pub(super) fn handle_l25_batch_d_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::CardsAsk
        && ctx.res.answer.as_ref().is_some_and(|a| a.is_empty())
    {
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ssr_fenchuan" => step_ssr_fenchuan(m, ctx),
        "ssr_newlihun" => step_ssr_newlihun(m, ctx),
        "ssr_biyue" => step_ssr_biyue(m, ctx),
        "ssr_yjixi" => step_ssr_yjixi(m, ctx),
        "ssr_yanji" => step_ssr_yanji(m, ctx),
        "ssr_cuandi" => step_ssr_cuandi(m, ctx),
        "ssr_yongsi" => step_ssr_yongsi(m, ctx),
        _ => handle_l25_batch_d_ext(m, ctx),
    }
}

fn step_ssr_fenchuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        set_flag(ctx.core, owner, "l25_fenchuan_unresp", 0);
        set_flag(ctx.core, owner, "l25_fenchuan_dmg_boost", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            let Some(&tgt) = m
                .targets
                .iter()
                .find(|&&t| t != owner && !ctx.core.player(t).hand.is_empty())
            else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_FENCHUAN_SHOW)
        }
        HostStage::SelectCard => {
            let tgt = m.target.unwrap_or(owner);
            let Some(ChoiceOption::Card(shown_cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![shown_cid],
                to: Vec::new(),
            });
            m.cards = vec![shown_cid];
            let shown_suit = ctx.catalog.effective_card_suit(ctx.core, tgt, shown_cid);
            let has_match = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .any(|&c| ctx.catalog.effective_card_suit(ctx.core, owner, c) == shown_suit);
            if !has_match {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                set_flag(ctx.core, owner, "l25_fenchuan_triggered", 1);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_FENCHUAN_DISCARD)
        }
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            set_flag(ctx.core, owner, "l25_fenchuan_triggered", 1);
            if let Some(ChoiceOption::Card(disc_cid)) = ctx.answer().first().copied() {
                if ctx.core.player(owner).hand.contains(&disc_cid) {
                    ctx.discard_from_hand(owner, disc_cid);
                    set_flag(ctx.core, owner, "l25_fenchuan_unresp", 1);
                    set_flag(ctx.core, owner, "l25_fenchuan_dmg_boost", 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newlihun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        set_flag(ctx.core, owner, "l25_lihun_zero_hand", 0);
        set_flag(ctx.core, owner, "l25_lihun_qianxing", 0);
        for p in alive_all(ctx.core, owner) {
            set_flag(ctx.core, p, "l25_lihun_controlled", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWLIHUN_GIVE),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWLIHUN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWLIHUN_GUESS)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let guess = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            let tgt = m.target.unwrap_or(owner);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newlihun");
            set_flag(ctx.core, owner, "l25_newlihun_used", cnt);
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            let actual_tag = card_type_tag(ctx.catalog, cid);
            if ctx.core.player(owner).hand.contains(&cid) && tgt != owner {
                ctx.give_hand_card(owner, tgt, cid);
            }
            if guess == actual_tag {
                set_flag(ctx.core, owner, "l25_lihun_zero_hand", 1);
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            set_flag(ctx.core, tgt, "l25_lihun_controlled", 1);
            set_flag(ctx.core, owner, "l25_lihun_qianxing", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_biyue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    let ez = empty_zone_count(ctx.core, owner);
    set_flag(ctx.core, owner, "l25_biyue_empty_zones", ez);
    set_flag(ctx.core, owner, "l25_biyue_trick_ward", 1);
    set_flag(ctx.core, owner, "mark:ssr_biyue", ez);
    if m.card == Some(111) && ez > 0 {
        set_flag(ctx.core, owner, "l25_biyue_drew", ez);
        ctx.start_draw(owner, ez);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_ssr_yjixi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tgt = m.target.unwrap_or(owner);
    if (m.card == Some(110) || m.card == Some(108)) && tgt == owner {
        set_flag(ctx.core, owner, "l25_yjixi_init", 1);
        let holder = alive_others(ctx.core, owner).first().copied().unwrap_or(0);
        set_flag(ctx.core, holder, "l25_yjixi_has_yuxi", 1);
        set_flag(ctx.core, holder, "l25_yuxi_treasure_active", 1);
        if holder != owner {
            set_flag(ctx.core, owner, "l25_yjixi_sha_plus1", 1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            if he_cards(ctx.core, tgt).is_empty() {
                return Flow::Done;
            }
            if he_cards(ctx.core, tgt).len() == 1 {
                let cid = he_cards(ctx.core, tgt)[0];
                gain_from_player(ctx, owner, tgt, cid);
                set_flag(ctx.core, owner, "l25_yjixi_triggered", 1);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_YJIXI_GAIN)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                gain_from_player(ctx, owner, tgt, cid);
                set_flag(ctx.core, owner, "l25_yjixi_triggered", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_yanji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(110) || m.card == Some(108) {
        let val = u32::from(owner) + 1;
        for p in alive_others(ctx.core, owner) {
            set_flag(ctx.core, p, "l25_yanji_taunted_by", val);
        }
        return Flow::Done;
    }
    if m.card == Some(111) {
        set_flag(ctx.core, owner, "l25_yanji_extra_target", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YANJI_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_YANJI_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if let Some(&cid) = m.cards.first() {
                discard_he_cards(ctx, owner, &[cid]);
            }
            set_flag(ctx.core, owner, "l25_yanji_extra_target", u32::from(tgt) + 1);
            set_flag(ctx.core, owner, "l25_yanji_triggered", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_cuandi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_CUANDI_CONFIRM),
        HostStage::SelectNature => {
            let agreed = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm) | Some(ChoiceOption::Tag(0))
            );
            if !agreed || he_cards(ctx.core, owner).is_empty() {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_CUANDI_DISCARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                discard_he_cards(ctx, owner, &[cid]);
                set_flag(ctx.core, owner, "l25_cuandi_triggered", 1);
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_yongsi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(108) && m.stage == HostStage::Act {
        let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
        set_flag(ctx.core, owner, "l25_yongsi_triggered", 1);
        ctx.start_draw(owner, lost);
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: 1,
        });
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    if m.card == Some(102) && m.stage == HostStage::Act {
        return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_YONGSI_CONFIRM);
    }
    match m.stage {
        HostStage::CompareTargetPick => {
            if !is_confirmed(ctx) {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_YONGSI_DISCARD)
        }
        HostStage::ComparePlayerPick => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if chosen.is_empty() {
                return Flow::Done;
            }
            discard_he_cards(ctx, owner, &chosen);
            set_flag(ctx.core, owner, "l25_yongsi_triggered", 1);
            ctx.start_draw(owner, 2);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YONGSI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_YONGSI_MODE)
        }
        HostStage::SelectNature => {
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            let tgt = m.target.unwrap_or(owner);
            if mode == 1 && !ctx.core.player(owner).hand.is_empty() {
                return wait_ask(m, ctx, HostStage::SelectCard, ASK_YONGSI_GIVE);
            }
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_yongsi");
            set_flag(ctx.core, owner, "l25_yongsi_used", cnt);
            if let Some(&cid) = ctx.core.player(tgt).hand.first() {
                ctx.discard_from_hand(tgt, cid);
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_yongsi");
            set_flag(ctx.core, owner, "l25_yongsi_used", cnt);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                if ctx.core.player(owner).hand.contains(&cid) && tgt != owner {
                    ctx.give_hand_card(owner, tgt, cid);
                }
                ctx.start_draw(owner, 2);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
