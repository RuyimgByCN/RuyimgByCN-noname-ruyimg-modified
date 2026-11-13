//! U08 英雄杀 Batch D 询问构造与前半部分步骤状态机（`u08_batch_d_asks.rs`）。

use super::super::*;
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

#[path = "u08_batch_d_ext.rs"]
mod u08_batch_d_ext;
use u08_batch_d_ext::handle_u08_batch_d_ext;

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

pub(super) fn answered_confirm(ctx: &Ctx<'_>) -> bool {
    ctx.answer().first() == Some(&ChoiceOption::Confirm)
}

pub(super) fn answered_tag(ctx: &Ctx<'_>) -> Option<u8> {
    ctx.answer().iter().find_map(|o| match o {
        ChoiceOption::Tag(t) => Some(*t),
        _ => None,
    })
}

pub(super) fn answered_player(ctx: &Ctx<'_>) -> Option<PlayerId> {
    ctx.answer().iter().find_map(|o| match o {
        ChoiceOption::Player(p) => Some(*p),
        _ => None,
    })
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

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u08_batch_d(
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
pub(super) fn build_spec(
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
    let (observer, mut spec) = match ask_id {
        ASK_BOLEHUIYAN_PREDICT => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "bolehuiyan:predict",
                options: vec![ChoiceOption::Tag(1), ChoiceOption::Tag(2)],
                min: 0,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_BOLEHUIYAN_DISCARD => {
            let options: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "bolehuiyan:discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_OLDBOLEHUIYAN_SUIT => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "oldbolehuiyan:predict_suit",
                options: vec![
                    ChoiceOption::Tag(1),
                    ChoiceOption::Tag(2),
                    ChoiceOption::Tag(3),
                    ChoiceOption::Tag(4),
                ],
                min: 0,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_YXSWEIYI_CONFIRM => Some((owner, ChoiceSpec::confirm("yxsweiyi:confirm"))),
        ASK_YXSWEIYI_SOURCE_DISCARD => {
            let src = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = hand_and_equips(core, src)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.len() < 2 {
                return None;
            }
            Some((
                src,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "yxsweiyi:source_discard",
                    options,
                    min: 2,
                    max: 2,
                    allow_cancel: false,
                },
            ))
        }
        ASK_QIANDU_CARD => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_black_card(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "qiandu:choose_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_QIANDU_TARGET => {
            let options: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "qiandu:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_NVQUAN1_RESPONSE => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c) == CardKind::Shan)
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "nvquan1:target_response",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_NVQUAN2_SHAN => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_sha_card(core, catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "nvquan2:target_shan",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_FEIGONG_DISCARD_SHA => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_sha_card(core, catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "feigong:discard_sha",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_JIANAI_CONFIRM => Some((owner, ChoiceSpec::confirm("jianai:confirm"))),
        ASK_JIEYONG_DISCARD_BLACK => {
            let used_cid = cards.first().copied();
            let options: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| Some(c) != used_cid && is_black_card(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "jieyong:discard_black",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_ZHULU_DISCARD_SAME_SUIT => {
            let trick_cid = cards.first().copied()?;
            let user = target.unwrap_or(owner);
            let trick_suit = catalog.effective_card_suit(core, user, trick_cid);
            let options: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| {
                    c != trick_cid && catalog.effective_card_suit(core, owner, c) == trick_suit
                })
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "zhulu:discard_same_suit",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_XIELING_DISCARD_TWO => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.len() < 2 {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "xieling:discard_two",
                    options,
                    min: 2,
                    max: 2,
                    allow_cancel: true,
                },
            ))
        }
        ASK_XIELING_CHOOSE_FROM => {
            let options: Vec<ChoiceOption> = xieling_from_candidates(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "xieling:choose_from",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_XIELING_CHOOSE_CARD => {
            let from = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = xieling_movable_cards(core, catalog, from)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "xieling:choose_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_XIELING_CHOOSE_TO => {
            let from = target.unwrap_or(owner);
            let move_cid = cards.first().copied()?;
            let options: Vec<ChoiceOption> =
                xieling_to_candidates(core, catalog, owner, from, move_cid)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "xieling:choose_to",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_QIANGYUN_CONFIRM => Some((owner, ChoiceSpec::confirm("qiangyun:confirm"))),
        ASK_CIKE_CONFIRM => Some((owner, ChoiceSpec::confirm("cike:confirm"))),
        ASK_CIKE_DISCARD_CARD => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = hand_and_equips(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "cike:discard_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_QIANGXI_TARGET => {
            let options: Vec<ChoiceOption> = qiangxi_targets(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "qiangxi:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_QIANGXI_COST => {
            let mut options = vec![ChoiceOption::Tag(1)];
            if !owner_weapon_cards(core, catalog, owner).is_empty() {
                options.push(ChoiceOption::Tag(2));
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "qiangxi:choose_cost",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_QIANGXI_WEAPON => {
            let options: Vec<ChoiceOption> = owner_weapon_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "qiangxi:choose_weapon",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_MIAOBI_TARGET => {
            let options: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !hand_and_equips(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "miaobi:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_MIAOBI_DISCARD_TARGET_CARD => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = hand_and_equips(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "miaobi:discard_target_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_MIAOBI_OWNER_DISCARD => {
            let options: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "miaobi:owner_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_MIAOBIX_CARD => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "miaobix:choose_card",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_MIAOBIX_TARGET => {
            let disc_cid = cards.first().copied()?;
            let use_cid = cards.get(1).copied().unwrap_or(disc_cid);
            let copy_kind = core
                .declared_kinds
                .get(&disc_cid)
                .copied()
                .unwrap_or_else(|| catalog.card_kind(disc_cid));
            let options: Vec<ChoiceOption> =
                miaobix_targets_for_kind(core, catalog, owner, copy_kind, use_cid)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "miaobix:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_ZHEXIAN_CONFIRM => Some((owner, ChoiceSpec::confirm("zhexian:confirm"))),
        ASK_GUIFU_TARGET => {
            let options: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).equips.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "guifu:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_GUIFU_EQUIP => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = core
                .player(t)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "guifu:choose_equip",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_LSHENGONG_HAND => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "lshengong:choose_hand",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_LSHENGONG_SOURCE => {
            let options: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).equips.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "lshengong:choose_source",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_LSHENGONG_EQUIP => {
            let src = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = core
                .player(src)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "lshengong:choose_equip",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_LSHENGONG_DEST => {
            let options: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "lshengong:choose_dest",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        _ => None,
    }?;
    if spec.allow_cancel && !spec.options.contains(&ChoiceOption::Pass) {
        spec.options.push(ChoiceOption::Pass);
    }
    Some((observer, spec))
}

pub(super) fn resolve_bolehuiyan_outcome(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    hit: bool,
) -> Flow {
    let owner = m.owner;
    let pred = flag(ctx.core, owner, "bolehuiyan_pred");
    set_flag(ctx.core, owner, "bolehuiyan_pred", 0);
    set_flag(ctx.core, owner, "bolehuiyan_attacker", 0);
    set_flag(ctx.core, owner, "bolehuiyan_target", 0);
    remove_skill_named(ctx, owner, "bolehuiyan2");
    remove_skill_named(ctx, owner, "bolehuiyan3");
    if pred == 0 {
        return Flow::Done;
    }
    let predicted_hit = pred == 1;
    if predicted_hit == hit {
        draw_direct(ctx, owner, 1);
        Flow::Done
    } else {
        let he = hand_and_equips(ctx.core, owner);
        if he.is_empty() {
            Flow::Done
        } else if he.len() == 1 {
            discard_cards(ctx, owner, &[he[0]]);
            Flow::Done
        } else {
            wait_ask(m, ctx, HostStage::SelectCard, ASK_BOLEHUIYAN_DISCARD)
        }
    }
}

pub(super) fn resolve_oldbolehuiyan_judge(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    j_owner: PlayerId,
    judge_cid: Option<CardId>,
    pred: u32,
) {
    set_flag(ctx.core, owner, "oldbolehuiyan_pred", 0);
    set_flag(ctx.core, owner, "oldbolehuiyan_target", 0);
    remove_skill_named(ctx, owner, "oldbolehuiyan2");
    if pred == 0 {
        return;
    }
    let actual_suit = judge_cid
        .filter(|&c| (c as usize) < ctx.catalog.cards.len())
        .map(|c| suit_code(ctx.catalog.effective_card_suit(ctx.core, j_owner, c)))
        .or_else(|| {
            let s = flag(ctx.core, j_owner, "last_judge_suit");
            if s > 0 {
                Some(s)
            } else {
                None
            }
        })
        .unwrap_or(1);
    if pred == actual_suit {
        draw_direct(ctx, owner, 2);
    } else if is_red_suit_code(pred) == is_red_suit_code(actual_suit) {
        draw_direct(ctx, owner, 1);
    }
}

pub(super) fn handle_u08_batch_d_impl(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "bolehuiyan" => match m.stage {
            HostStage::Act => match m.card {
                Some(1) => wait_ask(m, ctx, HostStage::SelectNature, ASK_BOLEHUIYAN_PREDICT),
                Some(2) => resolve_bolehuiyan_outcome(m, ctx, true),
                Some(3) => resolve_bolehuiyan_outcome(m, ctx, false),
                Some(4) => {
                    set_flag(ctx.core, owner, "bolehuiyan_used_turn", 0);
                    if flag(ctx.core, owner, "bolehuiyan_pred") > 0 {
                        return resolve_bolehuiyan_outcome(m, ctx, false);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                let Some(tag) = answered_tag(ctx) else {
                    return Flow::Done;
                };
                let pred = if tag == 2 { 2 } else { 1 };
                let attacker = m.targets.first().copied().unwrap_or(owner);
                let target = m.target.unwrap_or(owner);
                set_flag(ctx.core, owner, "bolehuiyan_used_turn", 1);
                set_flag(ctx.core, owner, "bolehuiyan_pred", pred);
                set_flag(ctx.core, owner, "bolehuiyan_attacker", u32::from(attacker) + 1);
                set_flag(ctx.core, owner, "bolehuiyan_target", u32::from(target) + 1);
                grant_skill_named(ctx, owner, "bolehuiyan2");
                grant_skill_named(ctx, owner, "bolehuiyan3");
                Flow::Done
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(cid) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, owner, &[cid]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bolehuiyan2" => match m.stage {
            HostStage::Act => resolve_bolehuiyan_outcome(m, ctx, false),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(cid) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, owner, &[cid]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bolehuiyan3" => match m.stage {
            HostStage::Act => resolve_bolehuiyan_outcome(m, ctx, true),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(cid) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, owner, &[cid]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "oldbolehuiyan" => match m.stage {
            HostStage::Act => match m.card {
                Some(1) => wait_ask(m, ctx, HostStage::SelectNature, ASK_OLDBOLEHUIYAN_SUIT),
                Some(2) => {
                    let pred = flag(ctx.core, owner, "oldbolehuiyan_pred");
                    let j_owner = m.target.unwrap_or(owner);
                    if pred > 0 {
                        resolve_oldbolehuiyan_judge(
                            ctx,
                            owner,
                            j_owner,
                            m.cards.first().copied(),
                            pred,
                        );
                        Flow::Done
                    } else {
                        wait_ask(
                            m,
                            ctx,
                            HostStage::ComparePlayerPick,
                            ASK_OLDBOLEHUIYAN_SUIT,
                        )
                    }
                }
                _ => Flow::Done,
            },
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if let Some(tag) = answered_tag(ctx) {
                    let suit_pred = u32::from(tag).clamp(1, 4);
                    let j_owner = m.target.unwrap_or(owner);
                    set_flag(ctx.core, owner, "oldbolehuiyan_pred", suit_pred);
                    set_flag(ctx.core, owner, "oldbolehuiyan_target", u32::from(j_owner) + 1);
                    grant_skill_named(ctx, owner, "oldbolehuiyan2");
                }
                Flow::Done
            }
            HostStage::ComparePlayerPick => {
                m.stage = HostStage::Act;
                if let Some(tag) = answered_tag(ctx) {
                    let suit_pred = u32::from(tag).clamp(1, 4);
                    let j_owner = m.target.unwrap_or(owner);
                    resolve_oldbolehuiyan_judge(
                        ctx,
                        owner,
                        j_owner,
                        m.cards.first().copied(),
                        suit_pred,
                    );
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "oldbolehuiyan2" => {
            let pred = flag(ctx.core, owner, "oldbolehuiyan_pred");
            if pred > 0 {
                let j_owner = m.target.unwrap_or(owner);
                resolve_oldbolehuiyan_judge(
                    ctx,
                    owner,
                    j_owner,
                    m.cards.first().copied(),
                    pred,
                );
            }
            Flow::Done
        }
        "xiangma" | "nvquan3" => Flow::Done,
        _ => handle_u08_batch_d_ext(m, ctx),
    }
}
