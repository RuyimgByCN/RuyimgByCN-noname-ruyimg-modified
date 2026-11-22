//! L01 `yiyi`（以逸待劳）多目标逐目标无懈窗口与摸二弃二结算机。

use super::*;
use crate::catalog::Catalog;
use crate::choice::ChoiceOption;
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;
use crate::vocabulary::host::L01_YIYI_DISCARD_PROMPT;

fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux.flags.get(&(p, key.to_string())).copied().unwrap_or(0)
}

fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

fn yiyi_nw_clear(core: &mut GameStateCore, owner: PlayerId, target: PlayerId) {
    let len_key = format!("l01_yiyi_nw_len:{owner}:{target}");
    let len = core.vm_aux.storage.remove(&len_key).unwrap_or(0);
    core.vm_aux
        .storage
        .remove(&format!("l01_yiyi_nw_pos:{owner}:{target}"));
    core.vm_aux
        .storage
        .remove(&format!("l01_yiyi_nw_null:{owner}:{target}"));
    for i in 0..len {
        core.vm_aux
            .storage
            .remove(&format!("l01_yiyi_nw_i:{owner}:{target}:{i}"));
    }
}

fn yiyi_nw_write(
    core: &mut GameStateCore,
    owner: PlayerId,
    target: PlayerId,
    nullified: u32,
    pos: u32,
    queue: &[PlayerId],
) {
    yiyi_nw_clear(core, owner, target);
    core.vm_aux
        .storage
        .insert(format!("l01_yiyi_nw_null:{owner}:{target}"), nullified);
    core.vm_aux
        .storage
        .insert(format!("l01_yiyi_nw_pos:{owner}:{target}"), pos);
    core.vm_aux
        .storage
        .insert(format!("l01_yiyi_nw_len:{owner}:{target}"), queue.len() as u32);
    for (i, &p) in queue.iter().enumerate() {
        core.vm_aux
            .storage
            .insert(format!("l01_yiyi_nw_i:{owner}:{target}:{i}"), u32::from(p));
    }
}

fn yiyi_nw_queue(core: &GameStateCore, catalog: &Catalog, start: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(start)
        .into_iter()
        .filter(|&p| {
            core.player(p).alive
                && !crate::kernel::response_kind_cards_core(
                    core,
                    catalog,
                    p,
                    crate::catalog::CardKind::Nullify,
                )
                .is_empty()
        })
        .collect()
}

fn yiyi_nw_spec(core: &GameStateCore, catalog: &Catalog, responder: PlayerId) -> ChoiceSpec {
    let mut options = vec![ChoiceOption::Pass];
    options.extend(
        crate::kernel::response_kind_cards_core(
            core,
            catalog,
            responder,
            crate::catalog::CardKind::Nullify,
        )
        .into_iter()
        .map(ChoiceOption::Card),
    );
    ChoiceSpec {
        kind: ChoiceKind::Confirm,
        prompt: "nullify_window",
        options,
        min: 1,
        max: 1,
        allow_cancel: false,
    }
}

fn yiyi_he_options(core: &GameStateCore, target: PlayerId) -> Vec<ChoiceOption> {
    let p = core.player(target);
    let mut opts: Vec<ChoiceOption> = p
        .hand
        .iter()
        .chain(p.equips.iter())
        .copied()
        .map(ChoiceOption::Card)
        .collect();
    opts.dedup();
    opts
}

fn finish_yiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(cid) = m.card.take() {
        if !ctx.core.discard.contains(&cid) {
            ctx.to_discard(cid);
        }
    }
    if ctx
        .core
        .vm_aux
        .storage
        .remove(&format!("l01_yiyi_any_ok:{}", m.owner))
        .unwrap_or(0)
        > 0
    {
        set_flag(ctx.core, m.owner, "l01_yiyi_resolved", 1);
    }
    Flow::Done
}

pub(super) fn step_l01_yiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let fengqi2 = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::HLSS_FENGQI2_KEY)
                .is_some_and(|d| ctx.core.has_skill(m.owner, d));
            while !m.targets.is_empty() {
                let t = m.targets.remove(0);
                if !ctx.core.player(t).alive {
                    continue;
                }
                if m.card.is_some() && !fengqi2 {
                    let queue = yiyi_nw_queue(ctx.core, ctx.catalog, m.owner);
                    if let Some(&resp) = queue.first() {
                        m.target = Some(t);
                        m.stage = HostStage::SelectNature;
                        yiyi_nw_write(ctx.core, m.owner, t, 0, 0, &queue);
                        return Flow::WaitChoice {
                            observer: resp,
                            spec: yiyi_nw_spec(ctx.core, ctx.catalog, resp),
                        };
                    }
                }
                m.target = Some(t);
                set_flag(ctx.core, m.owner, "d34_card_used:yiyi", 1);
                ctx.core
                    .vm_aux
                    .storage
                    .insert(format!("l01_yiyi_any_ok:{}", m.owner), 1);
                ctx.start_draw(t, 2);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            finish_yiyi(m, ctx)
        }
        HostStage::SelectNature => {
            let Some(t) = m.target else {
                m.stage = HostStage::Act;
                return Flow::Yield;
            };
            let len = ctx
                .core
                .vm_aux
                .storage
                .get(&format!("l01_yiyi_nw_len:{}:{t}", m.owner))
                .copied()
                .unwrap_or(0);
            let mut pos = ctx
                .core
                .vm_aux
                .storage
                .get(&format!("l01_yiyi_nw_pos:{}:{t}", m.owner))
                .copied()
                .unwrap_or(0);
            let mut nullified = ctx
                .core
                .vm_aux
                .storage
                .get(&format!("l01_yiyi_nw_null:{}:{t}", m.owner))
                .copied()
                .unwrap_or(0);
            let mut queue: Vec<PlayerId> = (0..len)
                .filter_map(|i| {
                    ctx.core
                        .vm_aux
                        .storage
                        .get(&format!("l01_yiyi_nw_i:{}:{t}:{i}", m.owner))
                        .copied()
                        .map(|p| p as PlayerId)
                })
                .collect();
            let responder = queue.get(pos as usize).copied();
            if let (Some(resp), Some(ChoiceOption::Card(cid))) =
                (responder, ctx.answer().first().copied())
            {
                crate::kernel::discard_response_card(
                    ctx,
                    resp,
                    cid,
                    crate::catalog::CardKind::Nullify,
                );
                ctx.log(crate::machine::LogEvent::CardResponded {
                    card: cid,
                    player: resp,
                    presentation: None,
                });
                nullified = 1 - nullified;
                queue = yiyi_nw_queue(ctx.core, ctx.catalog, resp);
                pos = 0;
            } else {
                pos = pos.saturating_add(1);
            }
            while (pos as usize) < queue.len() {
                let p = queue[pos as usize];
                if ctx.core.player(p).alive
                    && !crate::kernel::response_kind_cards_core(
                        ctx.core,
                        ctx.catalog,
                        p,
                        crate::catalog::CardKind::Nullify,
                    )
                    .is_empty()
                {
                    yiyi_nw_write(ctx.core, m.owner, t, nullified, pos, &queue);
                    return Flow::WaitChoice {
                        observer: p,
                        spec: yiyi_nw_spec(ctx.core, ctx.catalog, p),
                    };
                }
                pos += 1;
            }
            yiyi_nw_clear(ctx.core, m.owner, t);
            if nullified != 0 {
                if let Some(cid) = m.card {
                    ctx.log(crate::machine::LogEvent::CardNullified {
                        card: cid,
                        player: t,
                    });
                }
                m.target = None;
                m.stage = HostStage::Act;
                return step_l01_yiyi(m, ctx);
            }
            set_flag(ctx.core, m.owner, "d34_card_used:yiyi", 1);
            ctx.core
                .vm_aux
                .storage
                .insert(format!("l01_yiyi_any_ok:{}", m.owner), 1);
            ctx.start_draw(t, 2);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(t) = m.target else {
                m.stage = HostStage::Act;
                return Flow::Yield;
            };
            if !ctx.core.player(t).alive {
                m.target = None;
                m.stage = HostStage::Act;
                return step_l01_yiyi(m, ctx);
            }
            let options = yiyi_he_options(ctx.core, t);
            if options.is_empty() {
                m.target = None;
                m.stage = HostStage::Act;
                return step_l01_yiyi(m, ctx);
            }
            let need = options.len().min(2);
            m.stage = HostStage::TargetChoice;
            Flow::WaitChoice {
                observer: t,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: L01_YIYI_DISCARD_PROMPT,
                    options,
                    min: need,
                    max: need,
                    allow_cancel: false,
                },
            }
        }
        HostStage::TargetChoice => {
            if let Some(t) = m.target {
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let mut discarded = Vec::new();
                for c in picked {
                    if ctx.core.player(t).hand.contains(&c)
                        || ctx.core.player(t).equips.contains(&c)
                    {
                        ctx.take_owned_card(t, c);
                        ctx.to_discard(c);
                        discarded.push(c);
                    }
                }
                if !discarded.is_empty() {
                    let total =
                        flag(ctx.core, m.owner, "l01_yiyi_discarded") + discarded.len() as u32;
                    set_flag(ctx.core, m.owner, "l01_yiyi_discarded", total);
                    ctx.log(crate::machine::LogEvent::CardDiscarded {
                        player: t,
                        cards: discarded.clone(),
                    });
                    ctx.emit(EventData::CardsDiscarded {
                        player: t,
                        cards: discarded,
                    });
                }
            }
            m.target = None;
            m.stage = HostStage::Act;
            Flow::Yield
        }
        _ => finish_yiyi(m, ctx),
    }
}

pub(super) fn active_ask_l01_yiyi(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match stage {
        HostStage::SelectNature => {
            let t = target?;
            let pos = core
                .vm_aux
                .storage
                .get(&format!("l01_yiyi_nw_pos:{owner}:{t}"))
                .copied()?;
            let resp = core
                .vm_aux
                .storage
                .get(&format!("l01_yiyi_nw_i:{owner}:{t}:{pos}"))
                .copied()? as PlayerId;
            let spec = yiyi_nw_spec(core, catalog, resp);
            Some(ask(
                resp,
                spec.prompt,
                spec.kind,
                spec.options,
                spec.min,
                spec.max,
                spec.allow_cancel,
            ))
        }
        HostStage::TargetChoice => {
            let t = target?;
            let options = yiyi_he_options(core, t);
            let need = options.len().min(2);
            Some(ask(
                t,
                L01_YIYI_DISCARD_PROMPT,
                ChoiceKind::SelectCards,
                options,
                need,
                need,
                false,
            ))
        }
        _ => None,
    }
}
