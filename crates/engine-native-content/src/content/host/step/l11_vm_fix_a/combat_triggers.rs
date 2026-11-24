use super::super::super::*;
use super::super::l11_common::{alive_others, discard_player_cards, flag, he_cards, set_flag};
use super::helpers::*;
use crate::catalog::{CardKind, EquipSlot, Suit};
use crate::event::{DamageNature, EventData, Phase};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "fuji" => step_fuji(m, ctx, owner),
        "refankui" => step_refankui(m, ctx, owner),
        "fenwei" => step_fenwei(m, ctx, owner),
        "kuangfu" => step_kuangfu(m, ctx, owner),
        "wushen" => step_wushen(ctx, owner),
        "qiaomeng" => step_qiaomeng(m, ctx, owner),
        "shenxian" => step_simple_draw_confirm(m, ctx, owner, "shenxian", ASK_SHENXIAN_CONFIRM, Some("l11_shenxian_used")),
        "jiezi" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "jiezi",
                owner,
                presentation: None,
            });
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "bihuo" => step_bihuo(m, ctx, owner),
        "fenyin" => step_simple_draw_confirm(m, ctx, owner, "fenyin", ASK_FENYIN_CONFIRM, None),
        "jianying" => step_simple_draw_confirm(m, ctx, owner, "jianying", ASK_JIANYING_CONFIRM, None),
        "kaikang" => step_kaikang(m, ctx, owner),
        "lihuo" => step_lihuo(m, ctx, owner),
        "manji" => step_manji(m, ctx, owner),
        _ => Flow::Done,
    }
}

fn step_fuji(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Ok(src) = ctx.current_damage().map(|d| d.source) else {
                return Flow::Done;
            };
            let Some(src) = src.filter(|&s| ctx.core.player(s).alive) else {
                return Flow::Done;
            };
            if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                return Flow::Done;
            }
            m.target = Some(src);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_FUJI_CONFIRM)
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let Some(src) = m.target.filter(|&s| ctx.core.player(s).alive) else {
                return Flow::Done;
            };
            if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "fuji",
                owner,
                presentation: None,
            });
            ctx.core
                .vm_aux
                .flags
                .remove(&(src, "last_judge_card".to_string()));
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner: src,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(src) = m.target else {
                return Flow::Done;
            };
            let Some(jc) = ctx
                .core
                .vm_aux
                .flags
                .remove(&(src, "last_judge_card".to_string()))
            else {
                return Flow::Done;
            };
            let suit = ctx.catalog.effective_card_suit(ctx.core, src, jc);
            if matches!(suit, Suit::Spade | Suit::Club) {
                let _ = ctx.modify_damage_amount(1);
            } else if ctx.core.player(src).alive {
                let _ = give_single_discard_card_from_discard(ctx, src, jc);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_refankui(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let pool_of = |core: &crate::state::GameStateCore, src: PlayerId| {
        if src != owner {
            he_cards(core, src)
        } else {
            core.player(src).equips.clone()
        }
    };
    match m.stage {
        HostStage::Act | HostStage::ShuchuiTargetLoop => {
            if !owner_unblocked(ctx.core, ctx.catalog, owner, "refankui", false) {
                return Flow::Done;
            }
            let rem = m.card.unwrap_or(1);
            let Some(src) = m.target.filter(|&s| ctx.core.player(s).alive) else {
                return Flow::Done;
            };
            if rem == 0 || pool_of(ctx.core, src).is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_REFANKUI_GAIN)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(src) = m.target.filter(|&s| ctx.core.player(s).alive) else {
                return Flow::Done;
            };
            let Some(c) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|cid| pool_of(ctx.core, src).contains(cid))
            else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "refankui",
                owner,
                presentation: None,
            });
            give_he_cards_to_hand(ctx, src, owner, &[c]);
            let left = m.card.unwrap_or(1).saturating_sub(1);
            m.card = Some(left);
            if left > 0 && !pool_of(ctx.core, src).is_empty() {
                m.stage = HostStage::ShuchuiTargetLoop;
                Flow::Yield
            } else {
                Flow::Done
            }
        }
        _ => Flow::Done,
    }
}

fn step_fenwei(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Ok(u) = ctx.current_card_use() else {
                return Flow::Done;
            };
            let tgts = fenwei_effective_targets(ctx.core, ctx.catalog, u);
            if tgts.len() < 2 {
                return Flow::Done;
            }
            m.targets = tgts;
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_FENWEI_TARGETS)
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if ctx
                .catalog
                .skill_by_name("fenwei")
                .is_some_and(|d| ctx.core.skills.get(&(owner, d)).is_some_and(|i| i.once_used))
            {
                return Flow::Done;
            }
            let picks = answered_players(ctx);
            if picks.is_empty() {
                return Flow::Done;
            }
            mark_once_used(ctx, owner, "fenwei");
            ctx.log(LogEvent::SkillTriggered {
                skill: "fenwei",
                owner,
                presentation: None,
            });
            let fid = ctx.current_frame_id();
            let eff = ctx
                .current_card_use()
                .ok()
                .map(|u| u.effective_kind_in(ctx.core, ctx.catalog));
            let base = ctx
                .current_card_use()
                .ok()
                .map(|u| fenwei_effective_targets(ctx.core, ctx.catalog, u))
                .unwrap_or_default();
            let filtered: Vec<PlayerId> = base
                .into_iter()
                .filter(|p| !picks.contains(p))
                .collect();
            let u = ctx.current_card_use_mut();
            let u_player = u.player;
            u.targets = filtered;
            if u.targets.is_empty() {
                u.cancelled = true;
            } else if matches!(eff, Some(CardKind::Wugu | CardKind::Taoyuan)) {
                set_flag(ctx.core, u_player, &format!("dddanzhi_targets:{fid}"), 1);
                if eff == Some(CardKind::Taoyuan) {
                    set_flag(ctx.core, u_player, &format!("taoyuan_include_owner:{fid}"), 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_kuangfu(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_KUANGFU_EQUIP),
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let Some(c) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|cid| ctx.core.player(t).equips.contains(cid))
            else {
                return Flow::Done;
            };
            m.card = Some(c);
            let can_equip = ctx
                .catalog
                .card_slot_of(ctx.core, c)
                .is_some_and(|s| ctx.catalog.is_equip_slot_enabled(ctx.core, owner, s));
            if can_equip {
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_KUANGFU_EQUIP_SELF)
            } else {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "kuangfu",
                    owner,
                    presentation: None,
                });
                discard_player_cards(ctx, t, &[c]);
                Flow::Done
            }
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if let (Some(t), Some(c)) = (
                m.target.filter(|&p| ctx.core.player(p).alive),
                m.card.filter(|cid| {
                    m.target
                        .is_some_and(|p| ctx.core.player(p).equips.contains(cid))
                }),
            ) {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "kuangfu",
                    owner,
                    presentation: None,
                });
                if is_confirmed(ctx)
                    && ctx
                        .catalog
                        .card_slot_of(ctx.core, c)
                        .is_some_and(|s| ctx.catalog.is_equip_slot_enabled(ctx.core, owner, s))
                {
                    equip_stolen_card(ctx, t, owner, c);
                } else {
                    discard_player_cards(ctx, t, &[c]);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_wushen(ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    if ctx
        .current_card_use()
        .is_ok_and(|u| {
            !u.cancelled
                && u.player == owner
                && u.effective_kind_in(ctx.core, ctx.catalog).is_sha_like()
                && crate::kernel::card_use_suit(ctx.core, ctx.catalog, u) == Some(Suit::Heart)
        })
    {
        ctx.log(LogEvent::SkillTriggered {
            skill: "wushen",
            owner,
            presentation: None,
        });
    }
    Flow::Done
}

fn step_qiaomeng(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_QIAOMENG_DISCARD),
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let Some(c) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|cid| ctx.core.player(t).equips.contains(cid))
            else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "qiaomeng",
                owner,
                presentation: None,
            });
            m.card = Some(c);
            discard_player_cards(ctx, t, &[c]);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(c) = m.card.filter(|_| ctx.core.player(owner).alive) {
                if ctx.core.discard.contains(&c)
                    && matches!(
                        ctx.catalog.card_kind(c).equip_slot(),
                        Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
                    )
                {
                    let _ = give_single_discard_card_from_discard(ctx, owner, c);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_simple_draw_confirm(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    skill: &'static str,
    ask_id: u32,
    mark_flag: Option<&'static str>,
) -> Flow {
    match m.stage {
        HostStage::Act => {
            if let Some(fk) = mark_flag {
                if flag(ctx.core, owner, fk) > 0 {
                    return Flow::Done;
                }
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ask_id)
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if !is_confirmed(ctx) || !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            if let Some(fk) = mark_flag {
                if flag(ctx.core, owner, fk) > 0 {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, fk, 1);
            }
            ctx.log(LogEvent::SkillTriggered {
                skill,
                owner,
                presentation: None,
            });
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_bihuo(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_BIHUO_CONFIRM),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if !is_confirmed(ctx)
                || ctx
                    .catalog
                    .skill_by_name("bihuo")
                    .is_some_and(|d| ctx.core.skills.get(&(owner, d)).is_some_and(|i| i.once_used))
            {
                return Flow::Done;
            }
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            mark_once_used(ctx, owner, "bihuo");
            ctx.log(LogEvent::SkillTriggered {
                skill: "bihuo",
                owner,
                presentation: None,
            });
            let bonus = ctx.core.alive_players().count() as u32;
            set_flag(ctx.core, t, "l11_bihuo_distance", bonus);
            ctx.start_draw(t, 3);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_kaikang(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            while let Some(t) = m.targets.first().copied() {
                m.targets.remove(0);
                if ctx.core.player(t).alive
                    && ctx.core.computed_distance(ctx.catalog, owner, t) <= 1
                {
                    m.target = Some(t);
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_KAIKANG_CONFIRM);
                }
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if !is_confirmed(ctx) {
                m.target = None;
                m.stage = HostStage::Act;
                return step_kaikang(m, ctx, owner);
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "kaikang",
                owner,
                presentation: None,
            });
            ctx.start_draw(owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(t) = m.target.filter(|&p| p != owner && ctx.core.player(p).alive) else {
                m.target = None;
                m.stage = HostStage::Act;
                return step_kaikang(m, ctx, owner);
            };
            if he_cards(ctx.core, owner).is_empty() {
                m.target = None;
                m.stage = HostStage::Act;
                return step_kaikang(m, ctx, owner);
            }
            let _ = t;
            wait_ask(m, ctx, HostStage::SelectCard, ASK_KAIKANG_GIVE)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(c) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|cid| he_cards(ctx.core, owner).contains(cid))
            else {
                m.target = None;
                m.stage = HostStage::Act;
                return step_kaikang(m, ctx, owner);
            };
            let Some(t) = m.target.filter(|&p| p != owner && ctx.core.player(p).alive) else {
                m.target = None;
                m.stage = HostStage::Act;
                return step_kaikang(m, ctx, owner);
            };
            m.card = Some(c);
            give_he_cards_to_hand(ctx, owner, t, &[c]);
            m.stage = HostStage::JiefenOwnerDiscard;
            Flow::Yield
        }
        HostStage::JiefenOwnerDiscard => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                m.target = None;
                m.card = None;
                m.stage = HostStage::Act;
                return step_kaikang(m, ctx, owner);
            };
            let can_use_eq = m.card.is_some_and(|c| {
                ctx.core.player(t).hand.contains(&c)
                    && ctx
                        .catalog
                        .resolved_card_kind(ctx.core, c)
                        .equip_slot()
                        .is_some_and(|s| ctx.catalog.is_equip_slot_enabled(ctx.core, t, s))
            });
            if can_use_eq {
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_KAIKANG_USE_EQUIP)
            } else {
                m.target = None;
                m.card = None;
                m.stage = HostStage::Act;
                step_kaikang(m, ctx, owner)
            }
        }
        HostStage::ComparePlayerPick => {
            clear_ask(ctx, owner);
            let do_eq = is_confirmed(ctx);
            let t = m.target.take();
            let c = m.card.take();
            m.stage = HostStage::Act;
            if do_eq {
                if let (Some(t), Some(c)) = (t, c) {
                    if ctx.core.player(t).alive
                        && ctx.core.player(t).hand.contains(&c)
                        && ctx
                            .catalog
                            .resolved_card_kind(ctx.core, c)
                            .equip_slot()
                            .is_some_and(|s| ctx.catalog.is_equip_slot_enabled(ctx.core, t, s))
                    {
                        let next_fid = ctx.res.next_event_id;
                        if t == ctx.core.turn.current && ctx.core.turn.phase == Phase::Play {
                            set_flag(ctx.core, t, &format!("l11_play_equip_use:{next_fid}"), 1);
                        }
                        ctx.emit(EventData::Equipped {
                            player: t,
                            card: c,
                            replaced: None,
                        });
                        return Flow::Yield;
                    }
                }
            }
            step_kaikang(m, ctx, owner)
        }
        _ => Flow::Done,
    }
}

fn step_lihuo(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Ok(u) = ctx.current_card_use().cloned() else {
                return Flow::Done;
            };
            if u.player != owner || u.cancelled {
                return Flow::Done;
            }
            let eff = u.effective_kind_in(ctx.core, ctx.catalog);
            if eff == CardKind::Sha && u.nature == DamageNature::Normal {
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_LIHUO_CONVERT)
            } else if eff == CardKind::FireSha || u.nature == DamageNature::Fire {
                m.targets = u.targets;
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_LIHUO_EXTRA)
            } else {
                Flow::Done
            }
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let fid = ctx.current_frame_id();
            let card_opt;
            let targets_now;
            {
                let u = ctx.current_card_use_mut();
                u.nature = DamageNature::Fire;
                u.use_as = Some(CardKind::FireSha);
                card_opt = u.card;
                targets_now = u.targets.clone();
            }
            if let Some(cid) = card_opt {
                ctx.core.declared_kinds.insert(cid, CardKind::FireSha);
            }
            set_flag(ctx.core, owner, &format!("l11_lihuo_converted:{fid}"), 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "lihuo",
                owner,
                presentation: None,
            });
            let has_extra = alive_others(ctx.core, owner).into_iter().any(|p| {
                !targets_now.contains(&p)
                    && can_virtual_sha_target(ctx.core, ctx.catalog, owner, p, false)
            });
            if has_extra {
                m.targets = targets_now;
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_LIHUO_EXTRA)
            } else {
                Flow::Done
            }
        }
        HostStage::CompareTargetPick => {
            clear_ask(ctx, owner);
            let Some(extra) = answered_players(ctx).into_iter().next().filter(|&p| {
                !m.targets.contains(&p)
                    && can_virtual_sha_target(ctx.core, ctx.catalog, owner, p, false)
            }) else {
                return Flow::Done;
            };
            let u = ctx.current_card_use_mut();
            if !u.targets.contains(&extra) {
                u.targets.push(extra);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "lihuo",
                    owner,
                    presentation: None,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_manji(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Some(t) = m.target.filter(|&p| p != owner) else {
                return Flow::Done;
            };
            let oh = ctx.core.player(owner).hp;
            let th = ctx.core.player(t).hp;
            let damaged = oh < ctx.core.player(owner).max_hp as i32;
            if oh <= th && damaged {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "manji",
                    owner,
                    presentation: None,
                });
                ctx.recover(owner, 1, None);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            } else if oh >= th {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "manji",
                    owner,
                    presentation: None,
                });
                ctx.start_draw(owner, 1);
                Flow::Done
            } else {
                Flow::Done
            }
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target {
                if ctx.core.player(owner).alive
                    && ctx.core.player(owner).hp >= ctx.core.player(t).hp
                {
                    ctx.start_draw(owner, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
