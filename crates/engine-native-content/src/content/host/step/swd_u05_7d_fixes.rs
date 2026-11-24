//! U05-7d 杀海拾遗 swd 人物技能精确语义修复入口与判定/弃牌修正（`swd_u05_7d_fixes.rs`）。

use super::super::*;
use super::swd_u05_7d::{
    cancel_target_or_frame, discard_owned_card_once, finish_ordinary_judge, refund_once_used,
    start_ordinary_judge,
};
use engine_native_kernel::kernel::EloJudgePref;

pub(super) fn is_enemy_of(core: &crate::state::GameStateCore, owner: PlayerId, other: PlayerId) -> bool {
    if owner == other || !core.player(other).alive {
        return false;
    }
    match &core.mode {
        crate::state::GameMode::Identity { roles } => {
            use crate::state::Role;
            match (roles.get(owner as usize), roles.get(other as usize)) {
                (Some(Role::Lord | Role::Loyalist), Some(Role::Rebel | Role::Traitor)) => true,
                (Some(Role::Lord | Role::Loyalist), Some(Role::Lord | Role::Loyalist)) => false,
                (Some(Role::Rebel), Some(Role::Lord | Role::Loyalist)) => true,
                (Some(Role::Rebel), Some(Role::Rebel | Role::Traitor)) => false,
                (Some(Role::Traitor), _) => true,
                _ => true,
            }
        }
        crate::state::GameMode::Doudizhu { lord } => {
            if *lord == owner {
                other != owner
            } else {
                other == *lord
            }
        }
        _ => true,
    }
}

pub(super) fn he_cards(core: &crate::state::GameStateCore, p: PlayerId) -> Vec<CardId> {
    let mut v: Vec<CardId> = core.player(p).hand.iter().copied().collect();
    v.extend(core.player(p).equips.iter().copied());
    v
}

pub(super) fn is_host_enemy(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    other: PlayerId,
) -> bool {
    if !is_enemy_of(core, owner, other) {
        return false;
    }
    !matches!(core.source_is_enemy_of(catalog, owner, other), Some(false))
        || matches!(core.mode, crate::state::GameMode::Identity { .. })
}

pub(super) fn discard_random_he_card(ctx: &mut Ctx<'_>, target: PlayerId) -> Option<CardId> {
    let total = ctx.core.player(target).hand.len() + ctx.core.player(target).equips.len();
    if total == 0 {
        return None;
    }
    let j = ctx.core.rng.gen_range(total as u32) as usize;
    let hand_len = ctx.core.player(target).hand.len();
    if j < hand_len {
        let c = ctx.core.player(target).hand[j];
        // `discard_from_hand` already moves to discard, runs `on_hand_card_removed`, and logs `CardDiscarded`.
        ctx.discard_from_hand(target, c);
        Some(c)
    } else {
        let idx = j - hand_len;
        let c = ctx.core.player_mut(target).equips.remove(idx);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, c);
        ctx.to_discard(c);
        ctx.log(LogEvent::CardDiscarded {
            player: target,
            cards: vec![c],
        });
        Some(c)
    }
}

fn lqingcheng_flush_to_hand(ctx: &mut Ctx<'_>, m: &mut HostMachine) {
    let gained = std::mem::take(&mut m.cards);
    let mut moved = Vec::new();
    for c in gained {
        if let Some(pos) = ctx.core.discard.iter().position(|x| *x == c) {
            ctx.core.discard.remove(pos);
            ctx.core.player_mut(m.owner).hand.push(c);
            moved.push(c);
        }
    }
    if !moved.is_empty() {
        ctx.log(LogEvent::CardsDrawn {
            player: m.owner,
            cards: moved,
        });
    }
}

pub(super) fn step_swd_u05_7d_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::SwdShehun
        | HostBehavior::SwdXuanzhou
        | HostBehavior::SwdLexue
        | HostBehavior::SwdPozhen
        | HostBehavior::SwdGuiyan2 => super::swd_u05_7d_fixes_cards::step_swd_u05_7d_cards(m, ctx),
        HostBehavior::SwdXuehuang
        | HostBehavior::Moyan
        | HostBehavior::SwdHuajian
        | HostBehavior::Lxianglong
        | HostBehavior::SwdRexue
        | HostBehavior::Kunlunjing
        | HostBehavior::SwdDaixing3
        | HostBehavior::SwdYinguo
        | HostBehavior::SwdSusheng => super::swd_u05_7d_fixes_combat::step_swd_u05_7d_combat(m, ctx),
        HostBehavior::SwdBusi => match m.stage {
            HostStage::Act => {
                start_ordinary_judge(ctx, m.owner, Some(EloJudgePref::NonSpadePositive));
                m.stage = HostStage::SelectNature;
                Flow::Yield
            }
            HostStage::SelectNature => {
                let (suit, _, _) = finish_ordinary_judge(ctx, m.owner);
                if suit != crate::catalog::Suit::Spade && ctx.core.player(m.owner).alive {
                    let hp = ctx.core.player(m.owner).hp;
                    let need = (1 - hp).max(1) as u32;
                    ctx.recover(m.owner, need, None);
                    if !ctx.core.player(m.owner).face_down {
                        ctx.emit(EventData::TurnOverBefore {
                            player: m.owner,
                            cancelled: false,
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdHuanhun => match m.stage {
            HostStage::Act => {
                let Some(dp) = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .map(|f| &f.data)
                    .and_then(|data| match data {
                        EventData::Dying { player, .. } => Some(*player),
                        _ => None,
                    })
                else {
                    return Flow::Done;
                };
                m.target = Some(dp);
                let pool = he_cards(ctx.core, m.owner);
                if pool.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "huanhun_discard",
                        options: pool.into_iter().map(ChoiceOption::Card).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                }) else {
                    return Flow::Done;
                };
                let Some(dp) = m.target else {
                    return Flow::Done;
                };
                if !discard_owned_card_once(ctx, m.owner, picked) {
                    return Flow::Done;
                }
                m.card = Some(picked);
                start_ordinary_judge(ctx, dp, Some(EloJudgePref::RedPositive));
                m.stage = HostStage::SelectNature;
                Flow::Yield
            }
            HostStage::SelectNature => {
                let (Some(dp), Some(picked)) = (m.target, m.card) else {
                    return Flow::Done;
                };
                let (suit, _, _) = finish_ordinary_judge(ctx, dp);
                let red = matches!(suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
                if red {
                    if ctx.core.player(dp).alive || ctx.core.player(dp).hp <= 0 {
                        ctx.recover(dp, 1, None);
                    }
                } else if ctx.core.discard.contains(&picked) && ctx.core.player(dp).alive {
                    ctx.core.discard.retain(|x| *x != picked);
                    ctx.core.player_mut(dp).hand.push(picked);
                    ctx.log(LogEvent::CardsGiven {
                        from: m.owner,
                        to: dp,
                        cards: vec![picked],
                    });
                }
                m.target = None;
                m.card = None;
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdDunxing => match m.stage {
            HostStage::Act => {
                m.card = Some(ctx.current_event_frame_id() as CardId);
                let pool = he_cards(ctx.core, m.owner);
                if pool.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "dunxing_discard",
                        options: pool.into_iter().map(ChoiceOption::Card).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                }) else {
                    return Flow::Done;
                };
                if !discard_owned_card_once(ctx, m.owner, picked) {
                    return Flow::Done;
                }
                start_ordinary_judge(ctx, m.owner, Some(EloJudgePref::NonHeartPositive));
                m.stage = HostStage::SelectNature;
                Flow::Yield
            }
            HostStage::SelectNature => {
                let (suit, _, _) = finish_ordinary_judge(ctx, m.owner);
                if suit != crate::catalog::Suit::Heart {
                    if let Some(fid) = m.card.map(|v| v as crate::ids::EventId) {
                        cancel_target_or_frame(ctx, fid, m.owner);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdShending => match m.stage {
            HostStage::Act => {
                start_ordinary_judge(ctx, m.owner, Some(EloJudgePref::Shending));
                m.stage = HostStage::SelectNature;
                Flow::Yield
            }
            HostStage::SelectNature => {
                let (suit, _, _) = finish_ordinary_judge(ctx, m.owner);
                match suit {
                    crate::catalog::Suit::Heart => {
                        let pl = ctx.core.player(m.owner);
                        if pl.alive && pl.hp < pl.max_hp as i32 {
                            ctx.recover(m.owner, 1, None);
                        }
                    }
                    crate::catalog::Suit::Diamond => {
                        if ctx.core.player(m.owner).alive {
                            ctx.start_draw(m.owner, 1);
                        }
                    }
                    crate::catalog::Suit::Club => {
                        let mut pool: Vec<PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| {
                                *p != m.owner
                                    && is_enemy_of(ctx.core, m.owner, *p)
                                    && (!ctx.core.player(*p).hand.is_empty()
                                        || !ctx.core.player(*p).equips.is_empty())
                            })
                            .collect();
                        if !pool.is_empty() {
                            let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let t = pool.swap_remove(j);
                            let he = he_cards(ctx.core, t);
                            if !he.is_empty() {
                                let k = ctx.core.rng.gen_range(he.len() as u32) as usize;
                                discard_owned_card_once(ctx, t, he[k]);
                            }
                        }
                    }
                    crate::catalog::Suit::Spade => {}
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdHuanyin => match m.stage {
            HostStage::Act => {
                m.card = Some(ctx.current_event_frame_id() as CardId);
                start_ordinary_judge(ctx, m.owner, Some(EloJudgePref::Huanyin));
                m.stage = HostStage::SelectNature;
                Flow::Yield
            }
            HostStage::SelectNature => {
                let (suit, _, _) = finish_ordinary_judge(ctx, m.owner);
                match suit {
                    crate::catalog::Suit::Spade => {
                        if let Some(fid) = m.card.map(|v| v as crate::ids::EventId) {
                            cancel_target_or_frame(ctx, fid, m.owner);
                        }
                    }
                    crate::catalog::Suit::Heart => {
                        if ctx.core.player(m.owner).alive {
                            ctx.start_draw(m.owner, 1);
                        }
                    }
                    _ => {}
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdLqingcheng => loop {
            match m.stage {
                HostStage::Act => {
                    if !ctx.core.player(m.owner).alive {
                        lqingcheng_flush_to_hand(ctx, m);
                        return Flow::Done;
                    }
                    start_ordinary_judge(ctx, m.owner, Some(EloJudgePref::RedPositive));
                    m.stage = HostStage::SelectNature;
                    return Flow::Yield;
                }
                HostStage::SelectNature => {
                    let (suit, _, jcard) = finish_ordinary_judge(ctx, m.owner);
                    let red = matches!(
                        suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    );
                    if !red {
                        lqingcheng_flush_to_hand(ctx, m);
                        return Flow::Done;
                    }
                    if let Some(jc) = jcard {
                        m.cards.push(jc);
                    }
                    if m.cards.len() >= 3 {
                        lqingcheng_flush_to_hand(ctx, m);
                        return Flow::Done;
                    }
                    m.stage = HostStage::ComparePlayerPick;
                    return Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "lqingcheng_continue",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                HostStage::ComparePlayerPick => {
                    if ctx.answer().contains(&ChoiceOption::Confirm) {
                        m.stage = HostStage::Act;
                        continue;
                    }
                    lqingcheng_flush_to_hand(ctx, m);
                    return Flow::Done;
                }
                _ => return Flow::Done,
            }
        },
        HostBehavior::SwdDuanyi => match m.stage {
            HostStage::Act => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(m.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.len() < 2 {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "duanyi_discard",
                        options,
                        min: 2,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picks.len() < 2 {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.cards = picks;
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != m.owner && !ctx.catalog.has_undist(ctx.core, *p))
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "duanyi_target",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let cost = std::mem::take(&mut m.cards);
                for c in cost {
                    if ctx.core.player(m.owner).hand.contains(&c) {
                        ctx.discard_from_hand(m.owner, c);
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "断义",
                    owner: m.owner,
                    presentation: None,
                });
                m.target = Some(t);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(m.owner),
                    t,
                    1,
                    crate::event::DamageNature::Normal,
                    None,
                )));
                m.stage = HostStage::SelectNature;
                Flow::Yield
            }
            HostStage::SelectNature => {
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    let x = (ctx.core.player(t).max_hp as i32 - ctx.core.player(t).hp).max(0) as usize;
                    let mut pool = he_cards(ctx.core, t);
                    for _ in 0..x {
                        if pool.is_empty() {
                            break;
                        }
                        let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let c = pool.swap_remove(j);
                        discard_owned_card_once(ctx, t, c);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdToudan => loop {
            match m.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = he_cards(ctx.core, m.owner)
                        .into_iter()
                        .filter(|c| {
                            ctx.catalog.effective_card_suit(ctx.core, m.owner, *c)
                                == crate::catalog::Suit::Spade
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    }
                    m.stage = HostStage::SelectCard;
                    return Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "toudan_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    };
                    m.card = Some(c);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != m.owner && !ctx.catalog.has_undist(ctx.core, *p))
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    }
                    m.stage = HostStage::TargetChoice;
                    return Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "toudan_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    };
                    let Some(cost) = m.card.take() else {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive || !discard_owned_card_once(ctx, m.owner, cost) {
                        refund_once_used(ctx, m.owner, &m.def_key);
                        return Flow::Done;
                    }
                    m.target = Some(t);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(m.owner),
                        t,
                        1,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                    m.stage = HostStage::ComparePlayerPick;
                    return Flow::Yield;
                }
                HostStage::ComparePlayerPick => {
                    let Some(t) = m.target else {
                        return Flow::Done;
                    };
                    let mut queue = Vec::new();
                    if ctx.core.player(m.owner).alive {
                        queue.push(m.owner);
                    }
                    for p in ctx.core.seat_order_from(t) {
                        if ctx.core.player(p).alive
                            && ctx.core.computed_distance(ctx.catalog, t, p) <= 1
                        {
                            queue.push(p);
                        }
                    }
                    m.targets = queue;
                    m.stage = HostStage::ShuchuiTargetLoop;
                    continue;
                }
                HostStage::ShuchuiTargetLoop => {
                    while !m.targets.is_empty() {
                        let cur = m.targets.remove(0);
                        if !ctx.core.player(cur).alive {
                            continue;
                        }
                        let he = he_cards(ctx.core, cur);
                        if he.is_empty() {
                            continue;
                        }
                        m.target = Some(cur);
                        m.stage = HostStage::CardsAsk;
                        return Flow::WaitChoice {
                            observer: cur,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "toudan_neighbor_discard",
                                options: he.into_iter().map(ChoiceOption::Card).collect(),
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    return Flow::Done;
                }
                HostStage::CardsAsk => {
                    if let (Some(cur), Some(ChoiceOption::Card(c))) =
                        (m.target, ctx.answer().first().copied())
                    {
                        discard_owned_card_once(ctx, cur, c);
                    }
                    m.stage = HostStage::ShuchuiTargetLoop;
                    continue;
                }
                _ => return Flow::Done,
            }
        },
        HostBehavior::SwdGuanhu => match m.stage {
            HostStage::Act => {
                let Some(d) = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .map(|f| &f.data)
                    .and_then(|data| match data {
                        EventData::Damage(d) => Some(d.target),
                        _ => None,
                    })
                else {
                    return Flow::Done;
                };
                m.target = Some(d);
                let pool = he_cards(ctx.core, d);
                if pool.is_empty() {
                    return Flow::Done;
                }
                let max = if !ctx.core.player(d).hand.is_empty()
                    && !ctx.core.player(d).equips.is_empty()
                {
                    2
                } else {
                    1
                };
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "guanhu_discard",
                        options: pool.into_iter().map(ChoiceOption::Card).collect(),
                        min: 0,
                        max,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                if picks.is_empty() {
                    return Flow::Done;
                }
                if picks.len() == 2 {
                    let h = &ctx.core.player(t).hand;
                    let e = &ctx.core.player(t).equips;
                    let one_hand_one_equip =
                        (h.contains(&picks[0]) && e.contains(&picks[1]))
                            || (e.contains(&picks[0]) && h.contains(&picks[1]));
                    if !one_hand_one_equip {
                        let pool = he_cards(ctx.core, t);
                        let max = if !h.is_empty() && !e.is_empty() { 2 } else { 1 };
                        return Flow::WaitChoice {
                            observer: m.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "guanhu_discard",
                                options: pool.into_iter().map(ChoiceOption::Card).collect(),
                                min: 0,
                                max,
                                allow_cancel: true,
                            },
                        };
                    }
                }
                for c in picks {
                    discard_owned_card_once(ctx, t, c);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
