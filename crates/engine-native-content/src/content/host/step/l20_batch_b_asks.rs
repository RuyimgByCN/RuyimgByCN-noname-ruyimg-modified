//! L20 标记补充 Batch B 询问派生（`l20_batch_b_asks.rs`）：
//! 负责还原 `HostBehavior::L20MarkBatchB` 在挂起等待玩家输入时的 `ExpectedAsk`。

use super::*;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

#[allow(clippy::too_many_arguments)]
fn build_spec(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    if stage != HostStage::SelectCard && stage != HostStage::TargetChoice {
        return None;
    }
    let ask_code = card?;
    match ask_code {
        ASK_ZHENGSU_CHOOSE_MODE => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "zhengsu:choose_mode",
                options: vec![
                    ChoiceOption::Tag(0),
                    ChoiceOption::Tag(1),
                    ChoiceOption::Tag(2),
                ],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_ZHENGSU_CHOOSE_REWARD => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "zhengsu:choose_reward",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_ZHENYI_SPADE_CONFIRM => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "zhenyi_spade:choose_suit",
                options: vec![
                    ChoiceOption::Tag(0),
                    ChoiceOption::Tag(1),
                    ChoiceOption::Tag(2),
                    ChoiceOption::Tag(3),
                ],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_TSPOWEI_CHOOSE_ACTION => {
            let t = target.unwrap_or(owner);
            let mut options = Vec::new();
            if !core.player(owner).hand.is_empty() {
                options.push(ChoiceOption::Tag(0));
            }
            if core.player(owner).hp >= core.player(t).hp && !core.player(t).hand.is_empty() {
                options.push(ChoiceOption::Tag(1));
            }
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "tspowei:choose_action",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_QINGYU_DISCARD_TWO => {
            let options: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if options.len() < 2 {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "qingyu:discard_two",
                    options,
                    min: 2,
                    max: 2,
                    allow_cancel: false,
                },
            ))
        }
        ASK_TSPOWEI_CHOOSE_CARD => {
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
                    prompt: "tspowei:choose_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_FZ_LIEGONG_CONFIRM => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "fz_liegong:confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_FZ_NEW_LONGDAN_TARGET => {
            let cands: Vec<PlayerId> = if !targets.is_empty() {
                targets.to_vec()
            } else {
                alive_others(core, owner)
            };
            let options: Vec<ChoiceOption> = cands.into_iter().map(ChoiceOption::Player).collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "fz_new_longdan:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_FZ_NEW_TIEJI_CONFIRM => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "fz_new_tieji:confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_FZ_NEW_TIEJI_TARGET_DISCARD => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            if options.is_empty() {
                return None;
            }
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "fz_new_tieji:target_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_FZ_XINKUANGGU_MODE => {
            let options = if core.player(owner).hp < core.player(owner).max_hp as i32 {
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]
            } else {
                vec![ChoiceOption::Tag(1)]
            };
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "fz_xinkuanggu:choose_mode",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_REKUANSHI_TARGET => {
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
                    prompt: "rekuanshi:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_KUANSHI_TARGET => {
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
                    prompt: "kuanshi:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_RECHUHAI_TARGET => {
            let candidates: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            let options: Vec<ChoiceOption> =
                candidates.into_iter().map(ChoiceOption::Player).collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "rechuhai:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_RECHUHAI_PINDIAN_CARD => {
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
                    prompt: "rechuhai:pindian_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_RECHUHAI_TARGET_PINDIAN_CARD => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "rechuhai:target_pindian_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_XINFU_LINGREN_TARGET => {
            let cands: Vec<PlayerId> = if !targets.is_empty() {
                targets
                    .iter()
                    .copied()
                    .filter(|&p| core.player(p).alive)
                    .collect()
            } else {
                let mask = flag(core, owner, "xinfu_lingren_targets_mask");
                (0..core.players.len() as PlayerId)
                    .filter(|&p| ((mask >> p) & 1) == 1 && core.player(p).alive)
                    .collect()
            };
            let options: Vec<ChoiceOption> = cands.into_iter().map(ChoiceOption::Player).collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "xinfu_lingren:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_XINFU_LINGREN_GUESS_MASK => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "xinfu_lingren:guess_mask",
                options: (0..=7).map(ChoiceOption::Tag).collect(),
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_SBTIEJI_CONFIRM => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "sbtieji:confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_SBTIEJI_MOUYI_OWNER => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "sbtieji:mouyi_owner",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        ASK_SBTIEJI_MOUYI_TARGET => {
            let t = target.unwrap_or(owner);
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "sbtieji:mouyi_target",
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_SBTIEJI_TARGET_GIVE => {
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
                    prompt: "sbtieji:gain_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_SBDUANLIANG_TARGET => {
            let options: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !catalog.has_diaohulishan(core, p))
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "sbduanliang:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_SBDUANLIANG_MOUYI_OWNER => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "sbduanliang:mouyi_owner",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        ASK_SBDUANLIANG_MOUYI_TARGET => {
            let t = target.unwrap_or(owner);
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "sbduanliang:mouyi_target",
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_SBDUANLIANG_GAIN_CARD => {
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
                    prompt: "sbduanliang:gain_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_SHOUYE_TARGETS => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "shouye:confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_SHOUYE_MOUYI_OWNER => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "shouye:mouyi_owner",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        ASK_SHOUYE_MOUYI_TARGET => {
            let t = target.unwrap_or(owner);
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "shouye:mouyi_target",
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_XINHONGYAN_SUIT => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "xinhongyan:choose_suit",
                options: (0..=3).map(ChoiceOption::Tag).collect(),
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        ASK_XINFU_LANGXI_TARGET => {
            let owner_hp = core.player(owner).hp;
            let options: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).hp <= owner_hp)
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "xinfu_langxi:choose_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_CHONGZHU_CARD => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&cid| is_recastable_card(core, catalog, owner, cid))
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "_chongzhu:choose_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_TSPOWEI_GAIN_CARD => {
            let t = target.unwrap_or(owner);
            let options: Vec<ChoiceOption> = core
                .player(t)
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
                    prompt: "tspowei:gain_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        ASK_FZ_NEW_TIEJI_CHOOSE_SEAL => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "fz_new_tieji:choose_seal",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        _ => None,
    }
}

pub(super) fn filter_l20_extra_four(
    name: &str,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match name {
        "xinhongyan" => match event {
            EventData::OrdinaryJudge {
                owner: j_owner,
                invocation,
            } if *invocation != u32::MAX
                && flag(
                    ctx.state,
                    *j_owner,
                    &format!("judge_pre_finalized:{invocation}"),
                ) > 0 =>
            {
                let zhenyi_suit = flag(ctx.state, *j_owner, "zhenyi_judge_suit");
                if zhenyi_suit > 0 {
                    zhenyi_suit == 2
                } else {
                    let s = ctx
                        .catalog
                        .cards
                        .get(*invocation as usize)
                        .map(|c| c.suit)
                        .unwrap_or(Suit::Club);
                    s == Suit::Heart || (s == Suit::Spade && *j_owner == owner)
                }
            }
            _ => false,
        },
        "xinfu_langxi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && alive_others(ctx.state, owner)
                        .into_iter()
                        .any(|p| ctx.state.player(p).hp <= ctx.state.player(owner).hp)
            }
            _ => false,
        },
        "fz_gzpaoxiao" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && ctx.state.turn.current == owner
                    && is_sha_use(ctx.catalog, u)
            }
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn handle_l20_extra_four(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "xinhongyan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, ASK_XINHONGYAN_SUIT),
            HostStage::SelectCard => {
                let chosen = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Tag(t) => Some(*t),
                    _ => None,
                });
                if let Some(tag) = chosen {
                    // 上游 `['spade','heart','diamond','club']`：
                    // Tag(0)=♠(1), Tag(1)=♥(2), Tag(2)=♦(4), Tag(3)=♣(3)。
                    let suit_val = match tag {
                        0 => 1,
                        1 => 2,
                        2 => 4,
                        _ => 3,
                    };
                    let j_owner = m.target.unwrap_or(owner);
                    set_flag(ctx.core, j_owner, "xinhongyan_judge_suit", suit_val);
                    set_flag(ctx.core, owner, "xinhongyan_applied", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinfu_langxi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, ASK_XINFU_LANGXI_TARGET),
            HostStage::SelectCard => {
                let chosen = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                });
                let Some(t) = chosen else {
                    return Flow::Done;
                };
                let dmg = ctx.core.rng.gen_range(3) as i32;
                set_flag(ctx.core, owner, "xinfu_langxi_fired", 1);
                set_flag(ctx.core, owner, "xinfu_langxi_last_dmg", dmg as u32);
                if dmg > 0 {
                    emit_damage(ctx, Some(owner), t, dmg, DamageNature::Normal);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fz_gzpaoxiao" => {
            let stamp = (ctx.core.turn.round.max(1) << 8) | u32::from(ctx.core.turn.current);
            if flag(ctx.core, owner, "fz_gzpaoxiao_turn_stamp") != stamp {
                set_flag(ctx.core, owner, "fz_gzpaoxiao_turn_stamp", stamp);
                set_flag(ctx.core, owner, "fz_gzpaoxiao_sha_count", 0);
            }
            let cnt = flag(ctx.core, owner, "fz_gzpaoxiao_sha_count") + 1;
            set_flag(ctx.core, owner, "fz_gzpaoxiao_sha_count", cnt);
            if cnt == 2 {
                set_flag(ctx.core, owner, "fz_gzpaoxiao_second_sha_drawn", 1);
                draw_direct(ctx, owner, 1);
            }
            Flow::Done
        }
        "_chongzhu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, ASK_CHONGZHU_CARD),
            HostStage::SelectCard => {
                let chosen = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                });
                if let Some(cid) = chosen {
                    if ctx.core.player(owner).hand.contains(&cid) {
                        ctx.take_hand_card(owner, cid);
                        ctx.to_discard(cid);
                        ctx.emit(EventData::CardsLost {
                            player: owner,
                            cards: vec![cid],
                            origins: vec![LostOrigin::Hand],
                        });
                        set_flag(ctx.core, owner, "l20_chongzhu_recast", 1);
                        draw_direct(ctx, owner, 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l20_batch_b(
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

pub(super) fn wait_ask(m: &mut HostMachine, ctx: &mut Ctx<'_>, ask_code: u32) -> Flow {
    m.card = Some(ask_code);
    m.stage = HostStage::SelectCard;
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
        Flow::Done
    }
}
