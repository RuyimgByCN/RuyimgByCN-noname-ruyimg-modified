//! U08 英雄杀 Batch B 前半部分步骤状态机与询问构造（`u08_batch_b_asks.rs`）。

use super::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::content::standard::Gender;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

#[path = "u08_batch_b_ext.rs"]
mod u08_batch_b_ext;

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.stage = next_stage;
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_spec(
        m.stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        Flow::WaitChoice { observer, spec }
    } else {
        m.stage = HostStage::Act;
        Flow::Done
    }
}

pub(super) fn answered_cards(ctx: &Ctx<'_>) -> Vec<CardId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_player(ctx: &Ctx<'_>) -> Option<PlayerId> {
    ctx.answer().iter().find_map(|o| match o {
        ChoiceOption::Player(p) => Some(*p),
        _ => None,
    })
}

pub(super) fn answered_players(ctx: &Ctx<'_>) -> Vec<PlayerId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_tag(ctx: &Ctx<'_>) -> Option<u8> {
    ctx.answer().iter().find_map(|o| match o {
        ChoiceOption::Tag(t) => Some(*t),
        _ => None,
    })
}

pub(super) fn answered_confirm(ctx: &Ctx<'_>) -> bool {
    ctx.answer().first() == Some(&ChoiceOption::Confirm)
}

pub(super) fn handle_u08_batch_b_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "baihe" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_BAIHE_CARD),
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    refund_once_used(ctx, owner, "baihe");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_BAIHE_TARGET)
            }
            HostStage::TargetChoice => {
                let (Some(&cid), Some(t)) = (m.cards.first(), answered_player(ctx)) else {
                    refund_once_used(ctx, owner, "baihe");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(t);
                discard_cards(ctx, owner, &[cid]);
                if ctx.core.player(t).linked {
                    ctx.core.player_mut(t).linked = false;
                    let hlen = ctx.core.player(t).hand.len();
                    if hlen == 1 {
                        let hc = ctx.core.player(t).hand[0];
                        discard_cards(ctx, t, &[hc]);
                        m.stage = HostStage::Act;
                        Flow::Done
                    } else if hlen > 1 {
                        wait_ask(
                            m,
                            ctx,
                            HostStage::CompareTargetPick,
                            ASK_BAIHE_TARGET_DISCARD,
                        )
                    } else {
                        m.stage = HostStage::Act;
                        Flow::Done
                    }
                } else {
                    ctx.core.player_mut(t).linked = true;
                    draw_direct(ctx, t, 1);
                    m.stage = HostStage::Act;
                    Flow::Done
                }
            }
            HostStage::CompareTargetPick => {
                m.stage = HostStage::Act;
                if let (Some(t), Some(hc)) = (m.target, answered_cards(ctx).into_iter().next()) {
                    discard_cards(ctx, t, &[hc]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yinyang" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YINYANG_CARDS),
            HostStage::SelectCard => {
                let cards = answered_cards(ctx);
                if cards.len() != 2 {
                    refund_once_used(ctx, owner, "yinyang");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                m.cards = cards;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_YINYANG_TARGETS)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let need = alive_all(ctx.core, owner).len().min(3);
                let chosen = answered_players(ctx);
                if chosen.len() != need {
                    refund_once_used(ctx, owner, "yinyang");
                    return Flow::Done;
                }
                let cards = std::mem::take(&mut m.cards);
                discard_cards(ctx, owner, &cards);
                for p in alive_all(ctx.core, owner) {
                    if chosen.contains(&p) {
                        let next = !ctx.core.player(p).linked;
                        ctx.core.player_mut(p).linked = next;
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xiushen" => {
            if ctx
                .core
                .alive_players()
                .any(|p| ctx.core.player(p).linked)
            {
                draw_direct(ctx, owner, 2);
            }
            Flow::Done
        }
        "yanyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YANYI_CARD),
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    refund_once_used(ctx, owner, "yanyi");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if !is_black_card(ctx.core, ctx.catalog, owner, cid) {
                    refund_once_used(ctx, owner, "yanyi");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_YANYI_TARGET)
            }
            HostStage::TargetChoice => {
                let (Some(&cid), Some(t)) = (m.cards.first(), answered_player(ctx)) else {
                    refund_once_used(ctx, owner, "yanyi");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(t);
                discard_cards(ctx, owner, &[cid]);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_YANYI_SUIT)
            }
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                let tag = answered_tag(ctx).unwrap_or(0);
                let declared_suit = match tag {
                    0 => Suit::Spade,
                    1 => Suit::Heart,
                    2 => Suit::Club,
                    _ => Suit::Diamond,
                };
                let hand = ctx.core.player(t).hand.clone();
                let matched = hand
                    .iter()
                    .any(|&c| ctx.catalog.effective_card_suit(ctx.core, t, c) == declared_suit);
                if matched && ctx.core.player(t).alive {
                    set_flag(ctx.core, owner, "yanyi_damage_active", u32::from(t) + 1);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                } else {
                    set_flag(ctx.core, owner, "yanyi_damage_active", 0);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jiean" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_JIEAN_CONFIRM),
            HostStage::SelectNature => {
                set_flag(ctx.core, owner, "yanyi_damage_active", 0);
                if !answered_confirm(ctx) {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let Some(t) = m.target else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let lost =
                    (ctx.core.player(t).max_hp as i32 - ctx.core.player(t).hp).max(0) as u32;
                if lost == 0 {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let drawn = draw_direct(ctx, owner, lost);
                m.cards = drawn;
                if m.cards.is_empty() || alive_others(ctx.core, owner).is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_JIEAN_GIVE_CARD)
            }
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "jiean_pending_card", cid + 1);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_JIEAN_GIVE_TARGET)
            }
            HostStage::TargetChoice => {
                let raw_cid = flag(ctx.core, owner, "jiean_pending_card");
                set_flag(ctx.core, owner, "jiean_pending_card", 0);
                let Some(dest) = answered_player(ctx) else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if raw_cid > 0 {
                    let cid = raw_cid - 1;
                    if ctx.core.player(owner).hand.contains(&cid) {
                        move_cards_to_hand(ctx, owner, dest, &[cid]);
                    }
                    m.cards
                        .retain(|&c| c != cid && ctx.core.player(owner).hand.contains(&c));
                }
                if !m.cards.is_empty() && !alive_others(ctx.core, owner).is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_JIEAN_GIVE_CARD);
                }
                m.stage = HostStage::Act;
                Flow::Done
            }
            _ => Flow::Done,
        },
        "batu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_BATU_CONFIRM),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    let groups = count_distinct_alive_factions(ctx.core, ctx.catalog);
                    let hlen = ctx.core.player(owner).hand.len();
                    if groups > hlen {
                        draw_direct(ctx, owner, (groups - hlen) as u32);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "wumu" => {
            ctx.core.turn.sha_used_play_phase = false;
            Flow::Done
        }
        "ysheshen" => match m.stage {
            HostStage::Act => {
                let count = m.cards.first().copied().unwrap_or(1).max(1);
                set_flag(ctx.core, owner, "ysheshen_rem", count);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_YSHESHEN_CONFIRM)
            }
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    set_flag(ctx.core, owner, "ysheshen_rem", 0);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let cards = pop_deck_cards(ctx, 2);
                if cards.is_empty() {
                    set_flag(ctx.core, owner, "ysheshen_rem", 0);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                m.cards = cards;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_YSHESHEN_TARGET_1)
            }
            HostStage::TargetChoice => {
                let dest = answered_player(ctx).unwrap_or(owner);
                if !m.cards.is_empty() {
                    let c1 = m.cards.remove(0);
                    if ctx.core.player(dest).alive {
                        ctx.core.player_mut(dest).hand.push(c1);
                        ctx.emit(EventData::CardsDrawn {
                            player: dest,
                            cards: vec![c1],
                        });
                    } else {
                        ctx.to_discard(Some(c1));
                    }
                }
                if !m.cards.is_empty() && !alive_all(ctx.core, owner).is_empty() {
                    return wait_ask(
                        m,
                        ctx,
                        HostStage::CompareTargetPick,
                        ASK_YSHESHEN_TARGET_2,
                    );
                }
                let rem = flag(ctx.core, owner, "ysheshen_rem");
                if rem > 1 && ctx.core.player(owner).alive {
                    set_flag(ctx.core, owner, "ysheshen_rem", rem - 1);
                    return wait_ask(m, ctx, HostStage::SelectNature, ASK_YSHESHEN_CONFIRM);
                }
                set_flag(ctx.core, owner, "ysheshen_rem", 0);
                m.stage = HostStage::Act;
                Flow::Done
            }
            HostStage::CompareTargetPick => {
                let dest = answered_player(ctx).unwrap_or(owner);
                if !m.cards.is_empty() {
                    let c2 = m.cards.remove(0);
                    if ctx.core.player(dest).alive {
                        ctx.core.player_mut(dest).hand.push(c2);
                        ctx.emit(EventData::CardsDrawn {
                            player: dest,
                            cards: vec![c2],
                        });
                    } else {
                        ctx.to_discard(Some(c2));
                    }
                }
                let rem = flag(ctx.core, owner, "ysheshen_rem");
                if rem > 1 && ctx.core.player(owner).alive {
                    set_flag(ctx.core, owner, "ysheshen_rem", rem - 1);
                    return wait_ask(m, ctx, HostStage::SelectNature, ASK_YSHESHEN_CONFIRM);
                }
                set_flag(ctx.core, owner, "ysheshen_rem", 0);
                m.stage = HostStage::Act;
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yxschangnian" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YXSCHANGNIAN_TARGET),
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                if let Some(t) = answered_player(ctx) {
                    let all_cards = hej_cards(ctx.core, owner);
                    if !all_cards.is_empty() {
                        move_cards_to_hand(ctx, owner, t, &all_cards);
                    }
                    set_flag(ctx.core, t, "yxschangnian2_active", 1);
                    grant_skill_named(ctx, t, "yxschangnian2");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yxschangnian2" => {
            draw_direct(ctx, owner, 1);
            Flow::Done
        }
        "yxsciqiu" => {
            if m.card == Some(1) {
                for &t in &m.targets {
                    if ctx.core.player(t).alive
                        && ctx.core.player(t).hp == ctx.core.player(t).max_hp as i32
                    {
                        set_flag(ctx.core, owner, &format!("yxsciqiu_sha_active:{t}"), 1);
                    }
                }
                return Flow::Done;
            }
            if flag(ctx.core, owner, "yxsciqiu_removed") > 0 {
                return Flow::Done;
            }
            if let Some(t) = m.target {
                set_flag(ctx.core, owner, &format!("yxsciqiu_sha_active:{t}"), 0);
                if !ctx.core.player(t).alive
                    || ctx.core.player(t).hp != ctx.core.player(t).max_hp as i32
                {
                    return Flow::Done;
                }
                let mut new_amt = 2;
                with_current_damage_mut(ctx, |dm| {
                    dm.amount = dm.amount.saturating_add(1);
                    new_amt = dm.amount;
                });
                if ctx.core.player(t).hp <= new_amt {
                    if !ctx.core.turn.ciqiu_dying.contains(&t) {
                        ctx.core.turn.ciqiu_dying.push(t);
                    }
                    set_flag(ctx.core, owner, "yxsciqiu_removed", 1);
                    remove_skill_named(ctx, owner, "yxsciqiu");
                }
            }
            Flow::Done
        }
        _ => u08_batch_b_ext::handle_u08_batch_b_ext(m, ctx),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u08_batch_b(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) = build_spec(
        stage, owner, target, targets, card, cards, core, catalog,
    )?;
    Some(ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_spec(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let ask_id = card?;
    let build = |observer: PlayerId,
                 prompt: &'static str,
                 kind: ChoiceKind,
                 mut options: Vec<ChoiceOption>,
                 min: usize,
                 max: usize,
                 allow_cancel: bool|
     -> Option<(PlayerId, ChoiceSpec)> {
        if options.is_empty() {
            return None;
        }
        if allow_cancel && !options.contains(&ChoiceOption::Pass) {
            options.push(ChoiceOption::Pass);
        }
        Some((
            observer,
            ChoiceSpec {
                prompt,
                kind,
                options,
                min,
                max,
                allow_cancel,
            },
        ))
    };

    match ask_id {
        ASK_BAIHE_CARD => {
            let opts = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "baihe:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_BAIHE_TARGET => {
            let opts = alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "baihe:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_BAIHE_TARGET_DISCARD => {
            let t = target?;
            let opts = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            build(t, "baihe:target_discard", ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_YINYANG_CARDS => {
            let opts = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            build(owner, "yinyang:discard_two", ChoiceKind::SelectCards, opts, 2, 2, true)
        }
        ASK_YINYANG_TARGETS => {
            let all = alive_all(core, owner);
            let need = all.len().min(3);
            if need == 0 {
                return None;
            }
            let opts = all.into_iter().map(ChoiceOption::Player).collect();
            build(owner, "yinyang:choose_targets", ChoiceKind::SelectPlayers, opts, need, need, true)
        }
        ASK_YANYI_CARD => {
            let opts = he_cards(core, owner)
                .into_iter()
                .filter(|&c| is_black_card(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "yanyi:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_YANYI_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "yanyi:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_YANYI_SUIT => {
            let opts = (0..4).map(ChoiceOption::Tag).collect();
            build(owner, "yanyi:choose_suit", ChoiceKind::PickTag, opts, 1, 1, false)
        }
        ASK_JIEAN_CONFIRM => build(
            owner,
            "jiean:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_JIEAN_GIVE_CARD => {
            let opts = cards
                .iter()
                .copied()
                .filter(|&c| core.player(owner).hand.contains(&c))
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "jiean:give_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_JIEAN_GIVE_TARGET => {
            let opts = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            build(owner, "jiean:give_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_BATU_CONFIRM => build(
            owner,
            "batu:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_YSHESHEN_CONFIRM => build(
            owner,
            "ysheshen:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_YSHESHEN_TARGET_1 => {
            let opts = alive_all(core, owner).into_iter().map(ChoiceOption::Player).collect();
            build(owner, "ysheshen:choose_target_1", ChoiceKind::SelectPlayers, opts, 1, 1, false)
        }
        ASK_YSHESHEN_TARGET_2 => {
            let opts = alive_all(core, owner).into_iter().map(ChoiceOption::Player).collect();
            build(owner, "ysheshen:choose_target_2", ChoiceKind::SelectPlayers, opts, 1, 1, false)
        }
        ASK_YXSCHANGNIAN_TARGET => {
            let opts = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            build(owner, "yxschangnian:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_SANBANFU_CONFIRM => build(
            owner,
            "sanbanfu:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_SANBANFU_TARGET_SHAN => {
            let t = target?;
            let opts = target_shan_cards(core, catalog, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build(t, "sanbanfu:target_shan", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_BINGSHENG_CARDS => {
            let opts = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            build(owner, "bingsheng:discard_two", ChoiceKind::SelectCards, opts, 2, 2, true)
        }
        ASK_BINGSHENG_TARGET => {
            let my_hp = core.player(owner).hp;
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    let tp = core.player(p);
                    tp.hp > my_hp || (tp.hp < my_hp && tp.hp < tp.max_hp as i32)
                })
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "bingsheng:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_SHENTAN_CARD => {
            let opts = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            build(owner, "shentan:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_SHENTAN_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    !core.player(p).hand.is_empty()
                        && core.computed_distance(catalog, owner, p) <= 2
                })
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "shentan:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_SHENTAN_GAIN_CARD => {
            let t = target?;
            let opts = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            build(owner, "shentan:gain_card", ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_SHENTAN_GIVE_TARGET => {
            let opts = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            build(owner, "shentan:give_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HEQIN_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| player_gender(core, catalog, p) == Gender::Male)
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "heqin:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_TIEWAN_CARD => {
            let opts = he_cards(core, owner)
                .into_iter()
                .filter(|&c| {
                    is_red_card(core, catalog, owner, c)
                        && !valid_le_targets(core, catalog, owner, c).is_empty()
                })
                .map(ChoiceOption::Card)
                .collect();
            build(owner, "tiewan:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_TIEWAN_TARGET => {
            let cid = cards.first().copied().unwrap_or(0);
            let opts = valid_le_targets(core, catalog, owner, cid)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(owner, "tiewan:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        _ => None,
    }
}
