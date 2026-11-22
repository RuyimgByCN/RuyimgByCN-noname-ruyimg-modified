//! L11（太虚幻境）武将技能 Batch A（S29–S43：`l11_skills_a.rs`）：
//!
//! - `tx_mojian`、`tx_yushou`、`tx_moyany`、`tx_danshu`
//! - `txyirang`、`txayboji`、`txayhanji`、`tx_dqtianzi`
//! - `xinmingjian`、`xinmingjian2`、`txmazhan`、`txlieji`
//! - `diycuidu`、`diychunxiao`、`diyzhongdu`

use super::l11_common::*;
use super::l11_skills_a_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

fn wait_a_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_skills_a_ask(
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
        m.card = None;
        Flow::Done
    }
}

pub(in crate::content::host) fn trigger_meta_l11_skills_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "tx_mojian" | "tx_yushou" | "txyirang" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new()))
        }
        "tx_moyany" | "tx_danshu" => {
            Some(meta(EventKind::CardsLost, Timing::Normal, 1, Vec::new()))
        }
        "txayboji" => Some(meta(EventKind::CardUsed, Timing::Normal, 2, Vec::new())),
        "txayhanji" => Some(meta(
            EventKind::DamageBefore,
            Timing::Normal,
            2,
            Vec::new(),
        )),
        "tx_dqtianzi" => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            1,
            vec![EventKind::PhaseBegin],
        )),
        "xinmingjian" => Some(meta(EventKind::TurnEnded, Timing::Normal, 1, Vec::new())),
        "diycuidu" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "diychunxiao" | "diyzhongdu" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new()))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_skills_a(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_skills_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    if skill.name != "xinmingjian" {
        return None;
    }
    let state = ctx.state;
    if state.turn.phase != Phase::Play || !is_unblocked_l11_owner(state, ctx.catalog, owner) {
        return None;
    }
    let ts = turn_stamp(state);
    let used = if flag(state, owner, "l11_mingjian_ts") == ts {
        flag(state, owner, "l11_mingjian_cnt")
    } else {
        0
    };
    if used >= 2 || state.player(owner).hand.is_empty() || alive_others(state, owner).is_empty() {
        return None;
    }
    active_skill_machine("xinmingjian", HostBehavior::L11GeneralSkillsA, owner)
}

pub(in crate::content::host) fn filter_l11_skills_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "tx_mojian" | "tx_yushou" => {
            let EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.is_friend_of(catalog, owner, p))
        }
        "tx_moyany" => {
            let EventData::CardsLost { player, cards, .. } = event else {
                return false;
            };
            *player == owner
                && !cards.is_empty()
                && state.turn.current != owner
                && flag(state, owner, "l11_moyany_ing") == 0
                && !alive_others(state, owner).is_empty()
        }
        "tx_danshu" => {
            let EventData::CardsLost { player, cards, .. } = event else {
                return false;
            };
            *player == owner
                && !cards.is_empty()
                && state.turn.current != owner
                && state.player(owner).hp < state.player(owner).max_hp as i32
        }
        "txyirang" => {
            let EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && !yirang_nonbasic_cards(state, catalog, owner).is_empty()
                && !yirang_targets(state, owner).is_empty()
        }
        "txayboji" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player == owner
                && !u.cancelled
                && flag(state, owner, "l11_boji_ts") != turn_stamp(state)
                && is_sha_use(catalog, u)
                && u.targets.len() == 1
                && u.targets[0] != owner
                && !he_cards(state, owner).is_empty()
        }
        "txayhanji" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && d.target != owner
                && !d.prevented
                && d.amount > 0
                && flag(state, owner, "l11_hanji_ts") != turn_stamp(state)
        }
        "tx_dqtianzi" => {
            if flag(state, owner, "l11_dqtianzi_ts") == turn_stamp(state)
                || alive_others(state, owner).is_empty()
            {
                return false;
            }
            match event {
                EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
                EventData::Phase {
                    player,
                    phase: Phase::Play,
                    begin: true,
                } => *player == owner,
                _ => false,
            }
        }
        "diycuidu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && d.target != owner
                && !d.prevented
                && d.amount > 0
                && state.player(d.target).alive
                && !state.is_friend_of(catalog, owner, d.target)
                && !has_named_skill(state, catalog, d.target, "diyzhongdu")
        }
        "diychunxiao" => {
            let EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner && state.player(owner).hp < state.player(owner).max_hp as i32
        }
        "diyzhongdu" => {
            let EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_skills_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    if let EventData::Damage(d) = event {
        target = Some(d.target);
    } else if let EventData::CardUsed(u) = event {
        target = u.targets.first().copied();
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11GeneralSkillsA,
        owner,
        target,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_skills_a(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "tx_mojian" | "tx_yushou" => match m.stage {
            HostStage::Act => {
                let ask = if m.def_key == "tx_mojian" {
                    ASK_MOJIAN_CONFIRM
                } else {
                    ASK_YUSHOU_CONFIRM
                };
                wait_a_ask(m, ctx, HostStage::TargetChoice, ask)
            }
            HostStage::TargetChoice => {
                if !is_confirmed(ctx) {
                    return Flow::Done;
                }
                let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                    .collect();
                if !enemies.is_empty() {
                    let kind = if m.def_key == "tx_mojian" {
                        CardKind::ArcheryAttack
                    } else {
                        CardKind::SavageAttack
                    };
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: enemies,
                        nature: DamageNature::Normal,
                        use_as: Some(kind),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "tx_moyany" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "l11_moyany_ing", 1);
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::CompareTargetPick => {
                let suit = flag(ctx.core, owner, "last_judge_suit");
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                set_flag(ctx.core, owner, "last_judge_rank", 0);
                if suit == 2 || suit == 4 {
                    let others = alive_others(ctx.core, owner);
                    if others.is_empty() {
                        set_flag(ctx.core, owner, "l11_moyany_ing", 0);
                        return Flow::Done;
                    }
                    if others.len() == 1 {
                        m.stage = HostStage::DrawSettle;
                        emit_damage(
                            ctx,
                            Some(owner),
                            others[0],
                            2,
                            DamageNature::Fire,
                            None,
                        );
                        return Flow::Yield;
                    }
                    return wait_a_ask(m, ctx, HostStage::TargetChoice, ASK_MOYANY_TARGET);
                }
                set_flag(ctx.core, owner, "l11_moyany_ing", 0);
                Flow::Done
            }
            HostStage::TargetChoice => {
                if let Some(t) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| ctx.core.player(p).alive)
                {
                    m.stage = HostStage::DrawSettle;
                    emit_damage(ctx, Some(owner), t, 2, DamageNature::Fire, None);
                    return Flow::Yield;
                }
                set_flag(ctx.core, owner, "l11_moyany_ing", 0);
                Flow::Done
            }
            _ => {
                set_flag(ctx.core, owner, "l11_moyany_ing", 0);
                Flow::Done
            }
        },
        "tx_danshu" => match m.stage {
            HostStage::Act => {
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::CompareTargetPick => {
                let suit = flag(ctx.core, owner, "last_judge_suit");
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                set_flag(ctx.core, owner, "last_judge_rank", 0);
                if suit == 2 || suit == 4 {
                    ctx.recover(owner, 1, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txyirang" => match m.stage {
            HostStage::Act => wait_a_ask(m, ctx, HostStage::TargetChoice, ASK_YIRANG_TARGET),
            HostStage::TargetChoice => {
                if let Some(t) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| ctx.core.player(p).alive)
                {
                    let give = yirang_nonbasic_cards(ctx.core, ctx.catalog, owner);
                    if !give.is_empty() {
                        move_cards_to_hand(ctx, owner, t, &give);
                    }
                    let tmax = ctx.core.player(t).max_hp;
                    if tmax > ctx.core.player(owner).max_hp && !ctx.core.player(owner).infinite_hp()
                    {
                        ctx.core.player_mut(owner).max_hp = tmax;
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txayboji" => match m.stage {
            HostStage::Act => wait_a_ask(m, ctx, HostStage::TargetChoice, ASK_BOJI_CONFIRM),
            HostStage::TargetChoice => {
                if !is_confirmed(ctx) {
                    return Flow::Done;
                }
                let pool = he_cards(ctx.core, owner);
                if pool.is_empty() {
                    return Flow::Done;
                }
                if pool.len() == 1 {
                    set_flag(ctx.core, owner, "l11_boji_ts", turn_stamp(ctx.core));
                    discard_player_cards(ctx, owner, &[pool[0]]);
                    if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                        let th = ctx.core.player(t).hand.clone();
                        discard_player_cards(ctx, t, &th);
                    }
                    m.stage = HostStage::DrawSettle;
                    ctx.start_draw(owner, 1);
                    return Flow::Yield;
                }
                wait_a_ask(m, ctx, HostStage::SelectCard, ASK_BOJI_DISCARD)
            }
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if let Some(c) = chosen.into_iter().next() {
                    set_flag(ctx.core, owner, "l11_boji_ts", turn_stamp(ctx.core));
                    discard_player_cards(ctx, owner, &[c]);
                    if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                        let th = ctx.core.player(t).hand.clone();
                        discard_player_cards(ctx, t, &th);
                    }
                    m.stage = HostStage::DrawSettle;
                    ctx.start_draw(owner, 1);
                    return Flow::Yield;
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txayhanji" => {
            let mut bumped = false;
            with_current_damage_mut(ctx, |dm| {
                dm.amount += 1;
                bumped = true;
            });
            if bumped {
                set_flag(ctx.core, owner, "l11_hanji_ts", turn_stamp(ctx.core));
            }
            Flow::Done
        }
        "tx_dqtianzi" => match m.stage {
            HostStage::Act => wait_a_ask(m, ctx, HostStage::TargetChoice, ASK_DQTIANZI_TARGET),
            HostStage::TargetChoice => {
                let Some(t) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| ctx.core.player(p).alive)
                else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11_dqtianzi_ts", turn_stamp(ctx.core));
                m.target = Some(t);
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::CompareTargetPick => {
                let suit = flag(ctx.core, owner, "last_judge_suit");
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                set_flag(ctx.core, owner, "last_judge_rank", 0);
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                if suit == 4 {
                    if ctx.core.turn.current == t && ctx.core.turn.phase < Phase::Play {
                        ctx.core.turn.skip_play_phase = true;
                    } else {
                        set_flag(ctx.core, t, "l11_dqtianzi_skip_play_next", 1);
                    }
                    return Flow::Done;
                }
                if (suit == 1 || suit == 3) && !ctx.core.player(t).hand.is_empty() {
                    return wait_a_ask(m, ctx, HostStage::SelectCard, ASK_DQTIANZI_CARDS);
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    if !chosen.is_empty() {
                        move_cards_to_hand(ctx, t, owner, &chosen);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinmingjian" => match m.stage {
            HostStage::Act => wait_a_ask(m, ctx, HostStage::SelectCard, ASK_MINGJIAN_CARDS),
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if chosen.is_empty() {
                    return Flow::Done;
                }
                m.cards = chosen;
                wait_a_ask(m, ctx, HostStage::TargetChoice, ASK_MINGJIAN_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(t) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| ctx.core.player(p).alive)
                else {
                    return Flow::Done;
                };
                let cards = std::mem::take(&mut m.cards);
                move_cards_to_hand(ctx, owner, t, &cards);
                if let Some(mj2) = ctx.catalog.skill_by_name("xinmingjian2") {
                    ctx.add_skill_until_turn_end(t, mj2);
                }
                let ts = turn_stamp(ctx.core);
                let cur = if flag(ctx.core, owner, "l11_mingjian_ts") == ts {
                    flag(ctx.core, owner, "l11_mingjian_cnt")
                } else {
                    0
                };
                set_flag(ctx.core, owner, "l11_mingjian_ts", ts);
                set_flag(ctx.core, owner, "l11_mingjian_cnt", cur + 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txlieji" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "l11_lieji_ts", turn_stamp(ctx.core));
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 1);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                let (orig_cid, kind_raw) = if m.cards.len() >= 2 {
                    (Some(m.cards[0]), m.cards[1])
                } else {
                    (None, m.cards.first().copied().unwrap_or(0))
                };
                let is_huogong = orig_cid
                    .and_then(|c| ctx.catalog.cards.get(c as usize))
                    .is_some_and(|cd| cd.name == "huogong");
                let eff_kind = if let Some(cid) = orig_cid {
                    ctx.catalog.card_kind(cid)
                } else if kind_raw == 2 {
                    CardKind::FireSha
                } else if kind_raw == 3 {
                    CardKind::LeiSha
                } else {
                    CardKind::Sha
                };
                if is_huogong {
                    set_flag(ctx.core, owner, VIRTUAL_HUOGONG_FLAG, 1);
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![t],
                        nature: DamageNature::Fire,
                        use_as: Some(CardKind::Wuzhong),
                        cancelled: false,
                    }));
                } else if let Some(cid) = orig_cid {
                    let is_l11_trick = matches!(
                        ctx.catalog.cards.get(cid as usize).map(|cd| cd.name),
                        Some(
                            "txhj_zhijizhibi"
                                | "txhj_yuanjiao"
                                | "txhj_yiyi"
                                | "txhj_jiedao"
                                | "txhj_huoshaolianying"
                                | "txhj_diaohu"
                                | "txhj_caomu"
                                | "txhj_wangmeizhike"
                                | "txhj_shengdong"
                                | "txhj_shuiyan"
                                | "txhj_jiuchiroulin"
                        )
                    );
                    let nature = match eff_kind {
                        CardKind::FireSha => DamageNature::Fire,
                        CardKind::LeiSha => DamageNature::Thunder,
                        _ => DamageNature::Normal,
                    };
                    if is_l11_trick {
                        set_flag(ctx.core, owner, VIRTUAL_TRICK_CID_FLAG, cid + 1);
                        ctx.emit(EventData::CardUsed(CardUseData {
                            card: None,
                            player: owner,
                            targets: vec![t],
                            nature,
                            use_as: Some(eff_kind),
                            cancelled: false,
                        }));
                    } else if eff_kind.is_sha_like() {
                        ctx.emit(EventData::CardUsed(CardUseData {
                            card: None,
                            player: owner,
                            targets: vec![t],
                            nature,
                            use_as: Some(eff_kind),
                            cancelled: false,
                        }));
                    } else if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
                        ctx.core.discard.remove(pos);
                        ctx.core.table.push(cid);
                        ctx.emit(EventData::CardUsed(CardUseData {
                            card: Some(cid),
                            player: owner,
                            targets: vec![t],
                            nature,
                            use_as: None,
                            cancelled: false,
                        }));
                    } else {
                        ctx.emit(EventData::CardUsed(CardUseData {
                            card: None,
                            player: owner,
                            targets: vec![t],
                            nature,
                            use_as: Some(eff_kind),
                            cancelled: false,
                        }));
                    }
                } else {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![t],
                        nature: match eff_kind {
                            CardKind::FireSha => DamageNature::Fire,
                            CardKind::LeiSha => DamageNature::Thunder,
                            _ => DamageNature::Normal,
                        },
                        use_as: Some(eff_kind),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "diycuidu" => {
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                ensure_perm_skill(ctx, t, "diyzhongdu");
            }
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "diychunxiao" => {
            let cur = ctx.core.player(owner).hp;
            let max = ctx.core.player(owner).max_hp as i32;
            if cur < max && ctx.core.rng.gen_range(2) == 0 {
                ctx.recover(owner, (max - cur) as u32, None);
            }
            Flow::Done
        }
        "diyzhongdu" => match m.stage {
            HostStage::Act => {
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::CompareTargetPick => {
                let suit = flag(ctx.core, owner, "last_judge_suit");
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                set_flag(ctx.core, owner, "last_judge_rank", 0);
                if suit != 2 {
                    emit_damage(ctx, None, owner, 1, DamageNature::Normal, None);
                }
                if suit != 1 {
                    remove_named_skill(ctx, owner, "diyzhongdu");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
