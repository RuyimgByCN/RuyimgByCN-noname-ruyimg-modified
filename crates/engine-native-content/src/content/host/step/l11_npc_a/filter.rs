//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 触发过滤、机器构造与主动技资格校验（`filter.rs`）。

use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, Suit};
use crate::event::{DamageNature, DrawOutcome, EventData, Frame, LostOrigin, Phase};
use crate::ids::PlayerId;

fn can_aocai_out_of_turn(ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    ctx.state.turn.current != owner
        && ctx.state.player(owner).alive
        && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
}

fn is_sole_min_hand(state: &crate::state::GameStateCore, owner: PlayerId) -> bool {
    let my = state.player(owner).hand.len();
    state
        .alive_players()
        .all(|p| p == owner || state.player(p).hand.len() > my)
}

fn fenji_frame_has_other_actor(
    catalog: &crate::catalog::Catalog,
    state: &crate::state::GameStateCore,
    frames: &[Frame],
    victim: PlayerId,
) -> bool {
    // 非自己弃置（他人牌/他人技能导致的失去），或回合外非自我主动给出的弃置/夺取
    if frames.iter().rev().any(|f| match &f.data {
        EventData::CardUsed(u) => {
            u.player != victim
                && !matches!(
                    u.effective_kind(catalog),
                    CardKind::Duel | CardKind::SavageAttack | CardKind::ArcheryAttack
                )
        }
        EventData::Damage(d) => d.source.is_some_and(|s| s != victim),
        _ => false,
    }) {
        return true;
    }
    state.turn.current != victim
        && frames.iter().any(|f| {
            matches!(
                f.data,
                EventData::DrawPhase { player, .. } | EventData::Phase { player, .. } if player != victim
            )
        })
}

pub(in crate::content::host) fn filter_l11_npc_a_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    let name = skill.name.as_str();
    let is_locked = matches!(
        name,
        "buqu"
            | "dcnutao"
            | "dualside"
            | "guju"
            | "jili"
            | "jsrgzhenqiao"
            | "lixia"
            | "mjmouzhi"
            | "olzhuiji"
            | "rejinjiu"
            | "reqizhou"
    );
    let unblocked = if is_locked {
        is_locked_npc_a(ctx.state, ctx.catalog, owner)
    } else {
        is_unblocked_npc_a(ctx.state, ctx.catalog, owner)
    };
    if !unblocked {
        return false;
    }
    if matches!(
        name,
        "rexingluan" | "mozhi" // `rexingluan` is usable:1; handled per-arm below too
    ) && has_used_skill(ctx.state, ctx.catalog, owner, name)
        && name == "rexingluan"
    {
        return false;
    }

    match name {
        "aocai" => can_aocai_out_of_turn(ctx, owner),
        "buqu" => {
            let EventData::Dying { player, .. } = event else {
                return false;
            };
            let fid = frame.map_or(0, |f| f.id);
            *player == owner && flag(ctx.state, owner, &format!("l11_buqu_dying:{fid}")) == 0
        }
        "chenqing" => {
            let EventData::Dying { player, .. } = event else {
                return false;
            };
            flag(ctx.state, owner, "l11_chenqing_round") != ctx.state.turn.round
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && p != *player)
        }
        "olniepan" => {
            matches!(event, EventData::Dying { player, .. } if *player == owner)
                && flag(ctx.state, owner, "l11_olniepan_used") == 0
        }
        "rexingshang" => {
            let EventData::Death { player, .. } = event else {
                return false;
            };
            *player != owner
                && (!he_cards(ctx.state, *player).is_empty()
                    || ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32)
        }
        "dualside" => {
            matches!(event, EventData::TurnOvered { player, .. } if *player == owner)
                && flag(ctx.state, owner, "l11_dualside_over") == 0
        }
        "fenji" => {
            let EventData::CardsLost {
                player, origins, ..
            } = event
            else {
                return false;
            };
            ctx.state.player(*player).alive
                && origins.contains(&LostOrigin::Hand)
                && fenji_frame_has_other_actor(ctx.catalog, ctx.state, frames, *player)
        }
        "lirang" => {
            let cur_fid = frame.map_or(1, |f| f.id);
            !alive_others(ctx.state, owner).is_empty()
                && match event {
                    EventData::CardsDiscarded { player, cards } => {
                        *player == owner
                            && cards.iter().any(|&c| {
                                ctx.state.discard.contains(&c)
                                    && flag(ctx.state, owner, &format!("l11_discard_batched:{c}"))
                                        == cur_fid
                            })
                    }
                    EventData::CardsLost { player, cards, .. } => {
                        *player == owner
                            && cards.iter().any(|&c| {
                                ctx.state.discard.contains(&c)
                                    && flag(ctx.state, owner, &format!("l11_discard_batched:{c}"))
                                        == 0
                                    && (flag(ctx.state, owner, &format!("u01_discard_reason:{c}"))
                                        > 0
                                        || ctx.state.turn.phase == Phase::Discard)
                            })
                    }
                    _ => false,
                }
        }
        "relianying" => {
            if !ctx.state.player(owner).hand.is_empty() {
                return false;
            }
            let fid = frame.map_or(0, |f| f.id);
            if flag(ctx.state, owner, "l11_relianying_last_fid") == fid as u32 {
                return false;
            }
            let cur_fid = frame.map_or(1, |f| f.id);
            match event {
                EventData::CardsLost {
                    player,
                    cards,
                    origins,
                } => {
                    *player == owner
                        && origins.contains(&LostOrigin::Hand)
                        && cards.iter().any(|&c| {
                            flag(ctx.state, owner, &format!("l11_discard_batched:{c}")) == 0
                        })
                }
                EventData::CardUsed(u) => {
                    u.player == owner && flag(ctx.state, owner, &format!("l11_carduse_lost:{fid}")) > 0
                }
                EventData::CardsMoved { from, to, .. } => *from == owner && *from != *to,
                EventData::CardsDiscarded { player, cards } => {
                    *player == owner
                        && cards.iter().any(|&c| {
                            flag(ctx.state, owner, &format!("l11_discard_batched:{c}")) == cur_fid
                        })
                }
                _ => false,
            }
        }
        "retuntian" => {
            if ctx.state.turn.current == owner {
                return false;
            }
            let fid = frame.map_or(0, |f| f.id);
            if flag(ctx.state, owner, "l11_retuntian_last_fid") == fid as u32 {
                return false;
            }
            let cur_fid = frame.map_or(1, |f| f.id);
            match event {
                EventData::CardsLost {
                    player,
                    cards,
                    origins,
                } => {
                    *player == owner
                        && ((origins.contains(&LostOrigin::Hand)
                            && cards.iter().any(|&c| {
                                flag(ctx.state, owner, &format!("l11_discard_batched:{c}")) == 0
                            }))
                            || origins.contains(&LostOrigin::Equip))
                }
                EventData::CardsDiscarded { player, cards } => {
                    *player == owner
                        && cards.iter().any(|&c| {
                            flag(ctx.state, owner, &format!("l11_discard_batched:{c}")) == cur_fid
                        })
                }
                EventData::CardUsed(u) => {
                    u.player == owner && flag(ctx.state, owner, &format!("l11_carduse_lost:{fid}")) > 0
                }
                _ => false,
            }
        }
        "liangzhu" => {
            let EventData::Recover { player, amount, .. } = event else {
                return false;
            };
            *amount > 0
                && ctx.state.turn.phase == Phase::Play
                && *player == ctx.state.turn.current
                && ctx.state.player(*player).alive
        }
        "rexuanfeng" => match event {
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: false,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "l11_xuanfeng_discard_cnt") >= 2
                    && !alive_others(ctx.state, owner)
                        .into_iter()
                        .filter(|&p| !he_cards(ctx.state, p).is_empty())
                        .collect::<Vec<_>>()
                        .is_empty()
            }
            EventData::CardsLost {
                player, origins, ..
            } => {
                *player == owner
                    && origins.contains(&LostOrigin::Equip)
                    && alive_others(ctx.state, owner)
                        .into_iter()
                        .any(|p| !he_cards(ctx.state, p).is_empty())
            }
            _ => false,
        },
        "reqizhou" => match event {
            EventData::Equipped { player, .. } => *player == owner,
            EventData::CardsLost {
                player, origins, ..
            } => *player == owner && origins.contains(&LostOrigin::Equip),
            _ => false,
        },
        "jieyuan" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.prevented || d.amount <= 0 {
                return false;
            }
            let can_more = d.source == Some(owner)
                && d.target != owner
                && ctx.state.player(d.target).alive
                && ctx.state.player(d.target).hp >= ctx.state.player(owner).hp
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| !is_red_card(ctx.catalog, ctx.state, owner, c));
            let can_less = d.target == owner
                && d.source.is_some_and(|s| {
                    s != owner
                        && ctx.state.player(s).alive
                        && ctx.state.player(s).hp >= ctx.state.player(owner).hp
                })
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| is_red_card(ctx.catalog, ctx.state, owner, c));
            can_more || can_less
        }
        "rejinjiu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.prevented || d.amount <= 0 || d.target != owner {
                return false;
            }
            let Some(src) = d.source else {
                return false;
            };
            frames.iter().rev().any(|f| {
                matches!(&f.data, EventData::CardUsed(u) if u.player == src && u.effective_kind(ctx.catalog).is_sha_like() && flag(ctx.state, src, &format!("sha_used_jiu:{}", f.id)) > 0)
            })
        }
        "mjmouzhi" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.target != owner || d.card.is_none() {
                return false;
            }
            if d.settlement.committed_damage.is_none() {
                let recorded = flag(ctx.state, owner, "l11_mjmouzhi_suit");
                recorded > 0
                    && d.card.is_some_and(|c| {
                        suit_bit(card_suit(ctx.catalog, ctx.state, owner, c)) == recorded
                    })
            } else {
                !d.prevented && d.amount > 0
            }
        }
        "retianxiang" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && d.target == owner
                && !alive_others(ctx.state, owner).is_empty()
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| card_suit(ctx.catalog, ctx.state, owner, c) == Suit::Heart)
        }
        "repojun" => match event {
            EventData::CardUsed(u) => {
                !u.cancelled
                    && u.player == owner
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && u.targets.iter().any(|&t| {
                        ctx.state.player(t).alive
                            && ctx.state.player(t).hp > 0
                            && !he_cards(ctx.state, t).is_empty()
                    })
            }
            EventData::Damage(d) => {
                let via_sha = d
                    .card
                    .is_some_and(|c| ctx.catalog.resolved_card_kind(ctx.state, c).is_sha_like())
                    || frames.iter().rev().any(|f| {
                        matches!(&f.data, EventData::CardUsed(u) if u.player == owner && u.effective_kind(ctx.catalog).is_sha_like())
                    });
                !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && via_sha
                    && ctx.state.player(d.target).alive
                    && ctx.state.player(owner).hand.len() >= ctx.state.player(d.target).hand.len()
                    && ctx.state.player(owner).equips.len()
                        >= ctx.state.player(d.target).equips.len()
            }
            _ => false,
        },
        "benyu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            let Some(src) = d.source else {
                return false;
            };
            d.target == owner
                && d.amount > 0
                && !d.prevented
                && src != owner
                && ctx.state.player(src).alive
                && (ctx.state.player(owner).hand.len() < ctx.state.player(src).hand.len().min(5)
                    || he_cards(ctx.state, owner).len() > ctx.state.player(src).hand.len())
        }
        "chouce" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner && d.amount > 0 && !d.prevented
        }
        "renshe" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner && d.amount > 0 && !d.prevented
        }
        "guju" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target != owner
                && d.amount > 0
                && !d.prevented
                && flag(ctx.state, d.target, "l11_kui_mark") > 0
        }
        "requanji" => match event {
            EventData::Damage(d) => d.target == owner && d.amount > 0 && !d.prevented,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } => *player == owner && (ctx.state.player(owner).hand.len() as i32) > ctx.state.player(owner).hp,
            _ => false,
        },
        "qingxian" => {
            let any_dying = ctx.state.alive_players().any(|p| ctx.state.player(p).dying);
            match event {
                EventData::Damage(d) => {
                    d.target == owner
                        && d.amount > 0
                        && !d.prevented
                        && d.source.is_some_and(|s| s != owner && ctx.state.player(s).alive)
                }
                EventData::Recover { player, amount, .. } => {
                    *player == owner && *amount > 0 && !alive_others(ctx.state, owner).is_empty()
                }
                EventData::DyingSaved { .. } => {
                    !any_dying
                        && flag(ctx.state, owner, "l11_qingxian_queued") > 0
                        && !alive_others(ctx.state, owner).is_empty()
                }
                _ => false,
            }
        }
        "liyu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            let via_sha = d
                .card
                .is_some_and(|c| ctx.catalog.resolved_card_kind(ctx.state, c).is_sha_like())
                || frames.iter().rev().any(|f| {
                    matches!(&f.data, EventData::CardUsed(u) if u.player == owner && u.effective_kind(ctx.catalog).is_sha_like())
                });
            d.source == Some(owner)
                && d.target != owner
                && d.amount > 0
                && !d.prevented
                && ctx.state.player(d.target).alive
                && via_sha
                && !he_cards(ctx.state, d.target).is_empty()
                && ctx
                    .state
                    .alive_players()
                    .any(|p| p != d.target && p != owner)
        }
        "nsxianhai" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            let Some(src) = d.source else {
                return false;
            };
            src != owner
                && src == ctx.state.turn.current
                && ctx.state.player(src).alive
                && flag(ctx.state, owner, "l11_nsxianhai_round") != ctx.state.turn.round
                && (flag(ctx.state, src, "l11_source_dmg_turn") + d.amount.max(0) as u32)
                    > flag(ctx.state, owner, "l11_last_turn_dmg")
        }
        "duwu" => {
            let EventData::DyingSaved { player, .. } = event else {
                return false;
            };
            flag(ctx.state, owner, &format!("l11_duwu_dying_target:{}", player)) > 0
        }
        "bmcanshi" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            if u.cancelled || u.targets.len() != 1 {
                return false;
            }
            let k = u.effective_kind(ctx.catalog);
            let single = is_single_target_basic_or_trick(k);
            let cancel_incoming = u.player != owner
                && single
                && u.targets.contains(&owner)
                && flag(ctx.state, u.player, "l11_kui_mark") > 0;
            let add_extra = u.player == owner
                && single
                && ctx
                    .state
                    .alive_players()
                    .any(|p| !u.targets.contains(&p) && flag(ctx.state, p, "l11_kui_mark") > 0);
            cancel_incoming || add_extra
        }
        "dcmoukui" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player == owner
                && u.effective_kind(ctx.catalog).is_sha_like()
                && u.targets.iter().any(|&t| ctx.state.player(t).alive)
        }
        "dcnutao" => match event {
            EventData::CardUsed(u) => {
                !u.cancelled
                    && u.player == owner
                    && u.effective_kind(ctx.catalog).is_trick_kind()
                    && u.targets.iter().any(|&t| t != owner && ctx.state.player(t).alive)
            }
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && d.nature == DamageNature::Thunder
                    && d.amount > 0
                    && !d.prevented
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == Phase::Play
            }
            _ => false,
        },
        "gaoyuan" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player != owner
                && u.effective_kind(ctx.catalog).is_sha_like()
                && u.targets.contains(&owner)
                && !he_cards(ctx.state, owner).is_empty()
                && ctx.state.alive_players().any(|p| {
                    p != owner && p != u.player && flag(ctx.state, p, "l11_zhengjian_mark") > 0
                })
        }
        "jili" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            jili_can_trigger(ctx, owner, u)
        }
        "jinglve" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.card.is_some_and(|c| {
                flag(ctx.state, u.player, &format!("l11_jinglve_sishi_from:{owner}")) == c + 1
            })
        }
        "jsrgshichong" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player == owner
                && u.targets.len() == 1
                && u.targets[0] != owner
                && ctx.state.player(u.targets[0]).alive
                && !ctx.state.player(u.targets[0]).hand.is_empty()
        }
        "jsrgzhenqiao" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player == owner
                && u.effective_kind(ctx.catalog).is_sha_like()
                && !u.targets.is_empty()
                && flag(ctx.state, owner, "l11_zhenqiao_recast_guard") == 0
                && !ctx
                    .state
                    .player(owner)
                    .equips
                    .iter()
                    .any(|&c| ctx.catalog.card_slot_of(ctx.state, c) == Some(EquipSlot::Weapon))
        }
        "kyou_zhidian" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player == owner
                && u.effective_kind(ctx.catalog).is_sha_like()
                && !u.targets.is_empty()
        }
        "leiji" => match event {
            EventData::CardUsed(u) => {
                !u.cancelled
                    && u.player == owner
                    && u.effective_kind(ctx.catalog) == CardKind::Shan
                    && ctx.state.alive_players().next().is_some()
            }
            EventData::CardResponded { card, player } => {
                *player == owner
                    && ctx.catalog.resolved_card_kind(ctx.state, *card) == CardKind::Shan
                    && ctx.state.alive_players().next().is_some()
            }
            _ => false,
        },
        "olzhuiji" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player == owner
                && u.effective_kind(ctx.catalog).is_sha_like()
                && u.targets.iter().any(|&t| {
                    ctx.state.player(t).alive
                        && !he_cards(ctx.state, t).is_empty()
                        && ctx.state.computed_distance(ctx.catalog, owner, t) == 1
                })
        }
        "reduodao" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player != owner
                && u.effective_kind(ctx.catalog).is_sha_like()
                && u.targets.contains(&owner)
                && !he_cards(ctx.state, owner).is_empty()
                && ctx
                    .state
                    .player(u.player)
                    .equips
                    .iter()
                    .any(|&c| ctx.catalog.card_slot_of(ctx.state, c) == Some(EquipSlot::Weapon))
        }
        "reyajiao" => {
            if ctx.state.turn.current == owner {
                return false;
            }
            match event {
                EventData::CardUsed(u) => {
                    let fid = frame.map_or(0, |f| f.id);
                    !u.cancelled
                        && u.player == owner
                        && u.card.is_some()
                        && flag(ctx.state, owner, &format!("l11_carduse_lost:{fid}")) > 0
                }
                EventData::CardResponded { player, .. } => *player == owner,
                _ => false,
            }
        }
        "rezhijian" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && u.player == owner
                && u.effective_kind(ctx.catalog).is_equip()
        }
        "reluanji" => {
            let EventData::CardResponded { card, player } = event else {
                return false;
            };
            ctx.catalog.resolved_card_kind(ctx.state, *card) == CardKind::Shan
                && frames.iter().rev().any(|f| {
                    matches!(&f.data, EventData::CardUsed(u) if u.player == owner && u.effective_kind(ctx.catalog) == CardKind::ArcheryAttack && flag(ctx.state, owner, &format!("l11_reluanji_fid:{}", f.id)) > 0)
                })
                && ctx.state.player(*player).alive
        }
        "gnsheque" => {
            let EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } = event
            else {
                return false;
            };
            *player != owner
                && !ctx.state.player(*player).equips.is_empty()
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.resolved_card_kind(ctx.state, c).is_sha_like())
        }
        "jigong" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
        ),
        "lixia" => {
            let EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } = event
            else {
                return false;
            };
            *player != owner
                && ctx.state.player(*player).alive
                && !ctx.state.in_attack_range(ctx.catalog, *player, owner)
        }
        "mozhi" => {
            let EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && !ctx.state.player(owner).hand.is_empty()
                && flag(ctx.state, owner, "l11_mozhi_hist_len") > 0
        }
        "piaoling" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "reguanxing" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty()),
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "l11_reguanxing_end_ok") > 0
                    && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
            }
            _ => false,
        },
        "rejuece" => {
            let EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| flag(ctx.state, p, "l11_lost_card_turn") > 0)
        }
        "rezaiqi" => {
            let EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner && flag(ctx.state, owner, "l11_rezaiqi_red_discard_turn") > 0
        }
        "refenxun" => {
            let EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && ctx.state.alive_players().any(|t| {
                    flag(ctx.state, owner, &format!("l11_mod:fenxun_to:{t}")) > 0
                        && flag(ctx.state, owner, &format!("l11_refenxun_dealt:{t}")) == 0
                })
                && !he_cards(ctx.state, owner).is_empty()
        }
        "qingzhongx" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "l11_qingzhongx_armed") > 0
                    && !is_sole_min_hand(ctx.state, owner)
            }
            _ => false,
        },
        "diancai" => {
            let EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } = event
            else {
                return false;
            };
            let need = ctx.state.player(owner).hp.max(0) as u32;
            *player != owner
                && flag(ctx.state, owner, "l11_diancai_play_lost") >= need
                && (ctx.state.player(owner).hand.len() as u32) < ctx.state.player(owner).max_hp
        }
        "lihun" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } if *player == owner && flag(ctx.state, owner, "l11_lihun_target") > 0
        ),
        "nzry_kuizhu" => {
            let EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: false,
            } = event
            else {
                return false;
            };
            *player == owner && flag(ctx.state, owner, "l11_kuizhu_discard_cnt") > 0
        }
        "reluoyi" | "retuxi" => matches!(
            event,
            EventData::DrawPhase { player, outcome, pending, .. }
                if *player == owner && *outcome == DrawOutcome::Running && *pending > 0
        ),
        "reshuangxiong" => match event {
            EventData::DrawPhase {
                player,
                outcome,
                pending,
                ..
            } => *player == owner && *outcome == DrawOutcome::Running && *pending > 0,
            EventData::Damage(d) => {
                d.target == owner
                    && d.amount > 0
                    && !d.prevented
                    && ctx.state.player(owner).alive
                    && super::phase_turn::reshuangxiong_collectible_duel_shas(
                        ctx.catalog,
                        ctx.state,
                        frames,
                        owner,
                    )
                    .is_some()
            }
            EventData::CardResponded { player, .. } => {
                *player != owner
                    && ctx.state.player(owner).alive
                    && frames
                        .iter()
                        .rev()
                        .find_map(|f| match &f.data {
                            EventData::CardUsed(u) => Some(u),
                            _ => None,
                        })
                        .is_some_and(|u| {
                            u.effective_kind(ctx.catalog) == CardKind::Duel
                                && (u.player == owner || u.targets.contains(&owner))
                        })
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_npc_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match event {
        EventData::Dying { player, .. } | EventData::Death { player, .. } => {
            target = Some(*player);
        }
        EventData::Recover { player, .. } => {
            target = Some(*player);
        }
        EventData::Damage(d) => {
            target = d.source;
            if skill.name == "guju"
                || skill.name == "liyu"
                || skill.name == "rejinjiu"
                || skill.name == "repojun"
                || skill.name == "mjmouzhi"
            {
                target = Some(d.target);
            }
            cards.push(d.amount.max(0) as u32);
            cards.push(d.card.map_or(0, |c| c + 1));
            cards.push(match d.nature {
                DamageNature::Fire => 1,
                DamageNature::Thunder => 2,
                DamageNature::Normal => 0,
            });
            targets.push(d.target);
            if let Some(s) = d.source {
                targets.push(s);
            }
        }
        EventData::CardUsed(u) => {
            target = Some(u.player);
            if let Some(c) = u.card {
                cards.push(c);
            }
            targets = u.targets.clone();
        }
        EventData::CardResponded { card, player } => {
            target = Some(*player);
            cards.push(*card);
        }
        EventData::CardsLost {
            player,
            cards: lc,
            ..
        } => {
            target = Some(*player);
            cards = lc.clone();
        }
        EventData::CardsDiscarded { player, cards: dc } => {
            target = Some(*player);
            cards = dc.clone();
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            target = Some(*player);
            let p_code = match phase {
                Phase::Prepare => 1,
                Phase::Judge => 2,
                Phase::Draw => 3,
                Phase::Play => 4,
                Phase::Discard => 5,
                Phase::End => 6,
            };
            cards.push(p_code | (u32::from(*begin) << 8));
        }
        EventData::DrawPhase { player, pending, .. } => {
            target = Some(*player);
            cards.push((*pending).max(0) as u32);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11CrossNpcA,
        owner,
        target,
        card: None,
        cards,
        targets,
        stage: HostStage::Act,
    }
}
