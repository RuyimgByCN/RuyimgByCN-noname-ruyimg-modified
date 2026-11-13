//! U08 英雄杀 Batch C 询问构造与前半部分步骤状态机（`u08_batch_c_asks.rs`）。

use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.stage = next_stage;
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_spec(
        next_stage,
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

pub(super) fn answered_confirm(ctx: &Ctx<'_>) -> bool {
    ctx.answer().first().copied() == Some(ChoiceOption::Confirm)
}

pub(super) fn answered_cards(ctx: &Ctx<'_>) -> Vec<CardId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_players(ctx: &Ctx<'_>) -> Vec<PlayerId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_tag(ctx: &Ctx<'_>) -> Option<u8> {
    ctx.answer().iter().find_map(|o| match *o {
        ChoiceOption::Tag(t) => Some(t),
        _ => None,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn ask_u08_batch_c_impl(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) = build_spec(stage, owner, target, targets, card, cards, core, catalog)?;
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
    targets: &[PlayerId],
    card: Option<u32>,
    _cards: &[CardId],
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
        ASK_QIANGLUE_CONFIRM => build(
            owner,
            "qianglue:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_QIANGLUE_CARD => {
            let t = target?;
            let opts = he_cards(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "qianglue:choose_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            )
        }
        ASK_XIADAO_TARGET => {
            let t = target?;
            let opts = xiadao_candidates(core, owner, t)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(
                owner,
                "xiadao:confirm",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_XIADAO_CARD => {
            let t = target?;
            let opts = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "xiadao:choose_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            )
        }
        ASK_TONGYU_CARD => {
            let opts = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "tongyu:discard_cards",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_LZHANGYI_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(
                owner,
                "lzhangyi:discard_card",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_YIZHUANG_CARD => {
            let opts = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "yizhuang:discard_two",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_YIZHUANG_TARGET => {
            let opts = yizhuang_male_targets(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(
                owner,
                "yizhuang:choose_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_TUQIANG_CONFIRM => build(
            owner,
            "tuqiang:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_XUMOU_CONFIRM => build(
            owner,
            "xumou:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_ZHENSHA_CARDS => {
            let hand = &core.player(owner).hand;
            let has_jiu = hand.iter().any(|&c| catalog.card_kind(c) == CardKind::Jiu);
            let black_cnt = hand
                .iter()
                .filter(|&&c| is_black_card(core, catalog, owner, c))
                .count();
            let opts = zhensha_eligible_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let min = if has_jiu { 1 } else { 2 };
            let max = if black_cnt >= 2 { 2 } else { 1 };
            build(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                opts,
                min,
                max,
                true,
            )
        }
        ASK_DUCAI_CARD => {
            let opts = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "ducai:choose_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_FANPU_TARGETS => {
            let cands = fanpu_sha_targets(core, catalog, owner);
            let max = cands.len().min(3);
            let opts = cands.into_iter().map(ChoiceOption::Player).collect();
            build(
                owner,
                "fanpu:choose_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                max,
                true,
            )
        }
        ASK_FENGHUO_CARD => {
            let opts = core
                .player(owner)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "fenghuo:choose_equips",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_NICHANG_CONFIRM => build(
            owner,
            "nichang:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_FENGYAN_CONFIRM => build(
            owner,
            "fengyan:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_FENGYI_CARD => {
            let opts = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "fengyi:discard_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_FENGYI_TARGET => {
            let opts = alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(
                owner,
                "fengyi:choose_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_WANGE_CONFIRM => build(
            owner,
            "wange:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        ASK_WANGE2_TARGETS => {
            let cands = wange2_candidates(core, owner);
            let max_pick =
                (core.player(owner).max_hp as i32 - core.player(owner).hp).max(1) as usize;
            let max = cands.len().min(max_pick);
            let opts = cands.into_iter().map(ChoiceOption::Player).collect();
            build(
                owner,
                "wange:choose_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                max,
                true,
            )
        }
        ASK_SEYOU_TARGET => {
            let opts = alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build(
                owner,
                "seyou:choose_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_SEYOU_MALE_SHA => {
            let male = *targets.first()?;
            let opts = core
                .player(male)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_sha_card(core, catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            build(
                male,
                "respond_sha",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            )
        }
        ASK_SEYOU_GAIN_CARD => {
            let male = *targets.first()?;
            let opts = he_cards(core, male)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build(
                owner,
                "qianglue:choose_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            )
        }
        ASK_SHESHI_SUIT => {
            let opts = (0..4).map(ChoiceOption::Tag).collect();
            build(
                owner,
                "sheshi:choose_suit",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                true,
            )
        }
        _ => None,
    }
}

pub(super) fn clear_yizhuang_copied(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let copied_defs: Vec<u16> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter_map(|((p, k), &v)| {
            if *p == owner && v > 0 {
                k.strip_prefix("yizhuang_copied_def:")
                    .and_then(|s| s.parse::<u16>().ok())
            } else {
                None
            }
        })
        .collect();
    for def in copied_defs {
        ctx.core.skills.remove(&(owner, def));
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, format!("yizhuang_copied_def:{def}")));
    }
    set_flag(ctx.core, owner, "yizhuang_copied_count", 0);
    set_flag(ctx.core, owner, "yizhuang_target", 0);
    set_flag(ctx.core, owner, "yizhuang2_active", 0);
    remove_skill_named(ctx, owner, "yizhuang2");
}

pub(super) fn handle_u08_batch_c_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "qianglue" => match m.stage {
            HostStage::Act => {
                if m.card == Some(99) {
                    m.card = None;
                    let suit_val = flag(ctx.core, owner, "last_judge_suit");
                    set_flag(ctx.core, owner, "last_judge_suit", 0);
                    if suit_val == 1 || suit_val == 3 {
                        if let Some(t) = m.target {
                            let cards = he_cards(ctx.core, t);
                            if cards.len() == 1 {
                                move_cards_to_hand(ctx, t, owner, &[cards[0]]);
                            } else if cards.len() > 1 {
                                return wait_ask(m, ctx, HostStage::SelectCard, ASK_QIANGLUE_CARD);
                            }
                        }
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_QIANGLUE_CONFIRM)
            }
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                    return Flow::Done;
                }
                m.card = Some(99);
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(t) = m.target {
                    if let Some(cid) = answered_cards(ctx)
                        .into_iter()
                        .next()
                        .or_else(|| he_cards(ctx.core, t).into_iter().next())
                    {
                        move_cards_to_hand(ctx, t, owner, &[cid]);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "sheshu" => {
            let cid = m.cards.first().copied().unwrap_or(0);
            for &t in &m.targets {
                if ctx.core.player(t).alive && ctx.core.player(t).hp >= 3 {
                    set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{t}"), 1);
                    set_flag(ctx.core, owner, "yxs_sheshu_pierce", 1);
                }
            }
            Flow::Done
        }
        "xiadao" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_XIADAO_TARGET),
            HostStage::TargetChoice => {
                let Some(recipient) = answered_players(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                let Some(victim) = m.target else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.targets = vec![recipient];
                let hand = ctx.core.player(victim).hand.clone();
                if hand.is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                if hand.len() == 1 {
                    move_cards_to_hand(ctx, victim, recipient, &[hand[0]]);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_XIADAO_CARD)
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                let (Some(victim), Some(&recipient)) = (m.target, m.targets.first()) else {
                    return Flow::Done;
                };
                if let Some(cid) = answered_cards(ctx)
                    .into_iter()
                    .next()
                    .or_else(|| ctx.core.player(victim).hand.first().copied())
                {
                    move_cards_to_hand(ctx, victim, recipient, &[cid]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "lguiyin" => {
            if m.card == Some(2) {
                set_flag(ctx.core, owner, "tongyu_guiyin_used", 0);
                set_flag(ctx.core, owner, "skill_turn:tongyu_guiyin_used", 0);
                set_flag(ctx.core, owner, "turn_damage_dealt", 0);
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "tongyu_guiyin_used", 1);
            set_flag(ctx.core, owner, "skill_turn:tongyu_guiyin_used", 1);
            draw_direct(ctx, owner, 1);
            remove_skill_named(ctx, owner, "lguiyin");
            remove_skill_named(ctx, owner, "sheshu");
            remove_skill_named(ctx, owner, "xiadao");
            grant_skill_named(ctx, owner, "jimin");
            grant_skill_named(ctx, owner, "lzhangyi");
            grant_skill_named(ctx, owner, "tongyu");
            Flow::Done
        }
        "tongyu" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    set_flag(ctx.core, owner, "tongyu_guiyin_used", 0);
                    set_flag(ctx.core, owner, "skill_turn:tongyu_guiyin_used", 0);
                    set_flag(ctx.core, owner, "turn_damage_dealt", 0);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_TONGYU_CARD)
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    refund_once_used(ctx, owner, "tongyu");
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &[cid]);
                set_flag(ctx.core, owner, "tongyu_guiyin_used", 1);
                set_flag(ctx.core, owner, "skill_turn:tongyu_guiyin_used", 1);
                remove_skill_named(ctx, owner, "jimin");
                remove_skill_named(ctx, owner, "lzhangyi");
                remove_skill_named(ctx, owner, "tongyu");
                grant_skill_named(ctx, owner, "lguiyin");
                grant_skill_named(ctx, owner, "sheshu");
                grant_skill_named(ctx, owner, "xiadao");
                Flow::Done
            }
            _ => Flow::Done,
        },
        "lzhangyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_LZHANGYI_TARGET),
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let Some(t) = answered_players(ctx).into_iter().next() else {
                    return Flow::Done;
                };
                for &c in &m.cards {
                    if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yizhuang" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    clear_yizhuang_copied(ctx, owner);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_YIZHUANG_CARD)
            }
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_YIZHUANG_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let (Some(&cid), Some(t)) = (m.cards.first(), answered_players(ctx).into_iter().next()) else {
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &[cid]);
                clear_yizhuang_copied(ctx, owner);
                let target_defs: Vec<u16> = ctx
                    .core
                    .skills
                    .iter()
                    .filter_map(|(&(p, def), inst)| (p == t && inst.enabled).then_some(def))
                    .collect();
                let mut copied = 0u32;
                for def in target_defs {
                    if !ctx.core.has_skill(owner, def) {
                        ctx.core.skills.insert(
                            (owner, def),
                            crate::state::SkillInstance {
                                owner,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                        set_flag(ctx.core, owner, &format!("yizhuang_copied_def:{def}"), 1);
                        copied += 1;
                    }
                }
                set_flag(ctx.core, owner, "yizhuang_copied_count", copied);
                set_flag(ctx.core, owner, "yizhuang_target", u32::from(t) + 1);
                set_flag(ctx.core, owner, "yizhuang2_active", 1);
                grant_skill_named(ctx, owner, "yizhuang2");
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yizhuang2" => {
            clear_yizhuang_copied(ctx, owner);
            Flow::Done
        }
        "tuqiang" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_TUQIANG_CONFIRM),
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    draw_direct(ctx, owner, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xumou" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_XUMOU_CONFIRM),
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    draw_direct(ctx, owner, 3);
                    let flipped = {
                        let p = ctx.core.player_mut(owner);
                        p.face_down = !p.face_down;
                        p.face_down
                    };
                    ctx.emit(EventData::TurnOvered {
                        player: owner,
                        face_down: flipped,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhensha" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHENSHA_CARDS),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                let picked = answered_cards(ctx);
                if picked.is_empty() {
                    return Flow::Done;
                }
                let to_discard: Vec<CardId> = if picked.len() == 1
                    && ctx.catalog.card_kind(picked[0]) == CardKind::Jiu
                {
                    vec![picked[0]]
                } else if picked.len() >= 2
                    && is_black_card(ctx.core, ctx.catalog, owner, picked[0])
                    && is_black_card(ctx.core, ctx.catalog, owner, picked[1])
                {
                    vec![picked[0], picked[1]]
                } else if picked.len() == 1
                    && is_black_card(ctx.core, ctx.catalog, owner, picked[0])
                {
                    let second = ctx
                        .core
                        .player(owner)
                        .hand
                        .iter()
                        .copied()
                        .find(|&c| c != picked[0] && is_black_card(ctx.core, ctx.catalog, owner, c));
                    let Some(c2) = second else {
                        return Flow::Done;
                    };
                    vec![picked[0], c2]
                } else {
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &to_discard);
                if let Some(dying_p) = m.target {
                    if !ctx.core.turn.ciqiu_dying.contains(&dying_p) {
                        ctx.core.turn.ciqiu_dying.push(dying_p);
                    }
                    let has_dying_frame = ctx
                        .res
                        .stack
                        .iter()
                        .any(|f| matches!(f.data, EventData::Dying { player, .. } if player == dying_p));
                    if !has_dying_frame {
                        ctx.emit(EventData::Death {
                            player: dying_p,
                            killer: Some(owner),
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => u08_batch_c_ext::handle_u08_batch_c_ext(m, ctx),
    }
}
