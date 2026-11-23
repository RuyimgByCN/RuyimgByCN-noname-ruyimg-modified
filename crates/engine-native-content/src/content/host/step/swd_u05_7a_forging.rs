//! U05-7a 器族锻造回合结束触发（`*_equip1..5` 共 30 技，tag `1286`）——自 `swd_u05_7a` 拆出。

use super::*;
use super::swd_u05_7a::{discard_he_card, hand_or_equip, is_red_card};
use crate::catalog::{Catalog, QueryCtx};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask as build_ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(super) fn forging_matches(name: &str, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    if name.starts_with("cangchizhibi_equip") {
        ctx.state.alive_players().next().is_some()
    } else if name.starts_with("huanglinzhicong_equip") {
        ctx.state.player(owner).hujia == 0
            && hand_or_equip(ctx.state, owner)
                .into_iter()
                .any(|c| !is_red_card(ctx.catalog, ctx.state, owner, c))
    } else if name.starts_with("xuanwuzhihuang_equip") {
        let pl = ctx.state.player(owner);
        pl.hp < pl.max_hp as i32
            && hand_or_equip(ctx.state, owner)
                .into_iter()
                .any(|c| is_red_card(ctx.catalog, ctx.state, owner, c))
    } else if name.starts_with("zhuquezhizhang_equip") {
        let has_red = hand_or_equip(ctx.state, owner)
            .into_iter()
            .any(|c| is_red_card(ctx.catalog, ctx.state, owner, c));
        let my_hp = ctx.state.player(owner).hp;
        has_red
            && ctx
                .state
                .alive_players()
                .any(|t| t != owner && ctx.state.player(t).hp >= my_hp)
    } else if name.starts_with("baishouzhihu_equip") {
        ctx.state
            .alive_players()
            .any(|t| t != owner && !hand_or_equip(ctx.state, t).is_empty())
    } else {
        name.starts_with("qinglongzhigui_equip")
    }
}

pub(super) fn step_forging_end(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let name = m.def_key.as_str();
    if name.starts_with("qinglongzhigui_equip") {
        ctx.start_draw(m.owner, 1);
        return Flow::Done;
    }
    if name.starts_with("cangchizhibi_equip") {
        return match m.stage {
            HostStage::Act => {
                let opts: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .map(ChoiceOption::Player)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "u05_cangchizhibi_forging_target",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive {
                        let next = !ctx.core.player(t).linked;
                        ctx.emit(EventData::Linked {
                            player: t,
                            linked: next,
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    if name.starts_with("huanglinzhicong_equip") {
        return match m.stage {
            HostStage::Act => {
                if ctx.core.player(m.owner).hujia > 0 {
                    return Flow::Done;
                }
                let opts: Vec<ChoiceOption> = hand_or_equip(ctx.core, m.owner)
                    .into_iter()
                    .filter(|c| !is_red_card(ctx.catalog, ctx.core, m.owner, *c))
                    .map(ChoiceOption::Card)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "u05_huanglinzhicong_forging_discard",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_he_card(ctx, m.owner, c);
                    ctx.change_hujia(m.owner, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    if name.starts_with("xuanwuzhihuang_equip") {
        return match m.stage {
            HostStage::Act => {
                let p = ctx.core.player(m.owner);
                if p.hp >= p.max_hp as i32 {
                    return Flow::Done;
                }
                let opts: Vec<ChoiceOption> = hand_or_equip(ctx.core, m.owner)
                    .into_iter()
                    .filter(|c| is_red_card(ctx.catalog, ctx.core, m.owner, *c))
                    .map(ChoiceOption::Card)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectNature;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "u05_xuanwuzhihuang_forging_discard",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectNature => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_he_card(ctx, m.owner, c);
                    let p = ctx.core.player(m.owner);
                    if p.alive && p.hp < p.max_hp as i32 {
                        ctx.recover(m.owner, 1, None);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    if name.starts_with("zhuquezhizhang_equip") {
        return match m.stage {
            HostStage::Act => {
                let my_hp = ctx.core.player(m.owner).hp;
                let has_target = ctx
                    .core
                    .alive_players()
                    .any(|t| t != m.owner && ctx.core.player(t).hp >= my_hp);
                if !has_target {
                    return Flow::Done;
                }
                let opts: Vec<ChoiceOption> = hand_or_equip(ctx.core, m.owner)
                    .into_iter()
                    .filter(|c| is_red_card(ctx.catalog, ctx.core, m.owner, *c))
                    .map(ChoiceOption::Card)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::ComparePlayerPick;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "u05_zhuquezhizhang_forging_card",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::ComparePlayerPick => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.cards = vec![c];
                let my_hp = ctx.core.player(m.owner).hp;
                let opts: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|t| *t != m.owner && ctx.core.player(*t).hp >= my_hp)
                    .map(ChoiceOption::Player)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::CompareTargetPick;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "u05_zhuquezhizhang_forging_target",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::CompareTargetPick => {
                let Some(c) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                discard_he_card(ctx, m.owner, c);
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(m.owner),
                        t,
                        1,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    if name.starts_with("baishouzhihu_equip") {
        return match m.stage {
            HostStage::Act => {
                let opts: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|t| *t != m.owner && !hand_or_equip(ctx.core, *t).is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                if opts.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::ShuchuiTarget;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "u05_baishouzhihu_forging_target",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::ShuchuiTarget => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let pool = hand_or_equip(ctx.core, t);
                if !pool.is_empty() {
                    let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    discard_he_card(ctx, t, pool[idx]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    Flow::Done
}

pub(super) fn ask_forging(
    stage: HostStage,
    owner: PlayerId,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let cards_ask = |prompt: &'static str, opts: Vec<CardId>| {
        Some(build_ask(
            owner,
            prompt,
            ChoiceKind::SelectCards,
            opts.into_iter().map(ChoiceOption::Card).collect(),
            1,
            1,
            true,
        ))
    };
    let players_ask = |prompt: &'static str, opts: Vec<PlayerId>| {
        Some(build_ask(
            owner,
            prompt,
            ChoiceKind::SelectPlayers,
            opts.into_iter().map(ChoiceOption::Player).collect(),
            1,
            1,
            true,
        ))
    };
    match stage {
        HostStage::TargetChoice => {
            players_ask("u05_cangchizhibi_forging_target", core.alive_players().collect())
        }
        HostStage::SelectCard => {
            let opts = hand_or_equip(core, owner)
                .into_iter()
                .filter(|c| !is_red_card(catalog, core, owner, *c))
                .collect();
            cards_ask("u05_huanglinzhicong_forging_discard", opts)
        }
        HostStage::SelectNature => {
            let opts = hand_or_equip(core, owner)
                .into_iter()
                .filter(|c| is_red_card(catalog, core, owner, *c))
                .collect();
            cards_ask("u05_xuanwuzhihuang_forging_discard", opts)
        }
        HostStage::ComparePlayerPick => {
            let opts = hand_or_equip(core, owner)
                .into_iter()
                .filter(|c| is_red_card(catalog, core, owner, *c))
                .collect();
            cards_ask("u05_zhuquezhizhang_forging_card", opts)
        }
        HostStage::CompareTargetPick => {
            let my_hp = core.player(owner).hp;
            let opts = core
                .alive_players()
                .filter(|t| *t != owner && core.player(*t).hp >= my_hp)
                .collect();
            players_ask("u05_zhuquezhizhang_forging_target", opts)
        }
        HostStage::ShuchuiTarget => {
            let opts = core
                .alive_players()
                .filter(|t| *t != owner && !hand_or_equip(core, *t).is_empty())
                .collect();
            players_ask("u05_baishouzhihu_forging_target", opts)
        }
        _ => None,
    }
}
