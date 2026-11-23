//! U05-5b Batch A（19 技能 + 4 子技）：
//! `sqlongyin`, `sqlongnu`, `sqlonghuo`, `sqlongwu`, `jielue`, `gwhuanshuang`, `gwhuanshuang_disable`,
//! `gwjinli`, `gwjinli_jiu`, `gwliaotian`, `gwhuanbi`, `gwminxiang`, `gwlangshi`, `gwjingtian`,
//! `gwjingtian2`, `gwjingshi`, `gwweitu`, `gwweitu_gain`, `gwgouhun`, `gwfutian`, `gwzhongmo`,
//! `gwyewu`, `g_gw_yewu`。

use super::group_a_asks::*;
use super::group_a_steps::*;
use super::*;
use crate::catalog::TriggerMeta;
use crate::event::{DamageNature, EventKind, Timing};
use crate::state::SkillInstance;

fn meta(event: EventKind, extra_events: Vec<EventKind>) -> TriggerMeta {
    TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 0,
        order_selectable: false,
        optional: false,
        extra_events,
    }
}

pub(super) fn trigger_meta_group_a(name: &str) -> Option<TriggerMeta> {
    match name {
        "sqlongyin" => Some(meta(EventKind::TurnBegin, Vec::new())),
        "sqlongnu" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "sqlongwu" => Some(meta(EventKind::TurnPhaseEnd, vec![EventKind::TurnBegin])),
        "sqlonghuo" => Some(meta(EventKind::TurnBegin, Vec::new())),
        "jielue" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "gwhuanshuang" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnPhaseEnd])),
        "gwjinli_jiu" => Some(meta(EventKind::TurnPhaseEnd, Vec::new())),
        "gwlangshi" => Some(meta(EventKind::Damage, Vec::new())),
        "gwweitu" => Some(meta(EventKind::CardsDiscarded, Vec::new())),
        "gwfutian" => Some(meta(EventKind::DamageBefore, vec![EventKind::PhaseBegin])),
        "g_gw_yewu" => Some(meta(EventKind::TurnPhaseAfter, Vec::new())),
        _ => None,
    }
}

pub(super) fn filter_group_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "sqlongyin" | "sqlongwu" | "sqlonghuo" if matches!(event, EventData::TurnBegin { .. }) => {
            let EventData::TurnBegin { player } = *event else {
                return false;
            };
            if player != owner {
                return false;
            }
            let turn_stamp = state.turn.round.saturating_mul(16) + u32::from(owner) + 1;
            flag(state, owner, "gw_sqlongyin_done_turn") != turn_stamp
                && (has_named_skill(state, ctx.catalog, owner, "sqlongwu")
                    || has_named_skill(state, ctx.catalog, owner, "sqlongnu")
                    || has_named_skill(state, ctx.catalog, owner, "sqlonghuo"))
        }
        "sqlongnu" => {
            let EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } = *event
            else {
                return false;
            };
            if player != owner || state.deck.is_empty() {
                return false;
            }
            if has_named_skill(state, ctx.catalog, owner, "sqlongyin") {
                let turn_stamp = state.turn.round.saturating_mul(16) + u32::from(owner) + 1;
                if flag(state, owner, "gw_sqlongyin_done_turn") != turn_stamp {
                    return false;
                }
            }
            true
        }
        "sqlongwu" => matches!(
            *event,
            EventData::TurnEndWindow {
                player,
                after: false
            } if player == owner && !state.player(owner).hand.is_empty()
        ),
        "jielue" => {
            let EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } = *event
            else {
                return false;
            };
            if player != owner {
                return false;
            }
            friends_of(state, ctx.catalog, owner).into_iter().any(|f| {
                state
                    .player(f)
                    .hand
                    .iter()
                    .any(|&c| b5b_card_can_use(state, ctx.catalog, owner, c, false))
            })
        }
        "gwhuanshuang" => {
            if has_named_skill(state, ctx.catalog, owner, "gwhuanshuang_disable") {
                return false;
            }
            let ok_phase = matches!(
                *event,
                EventData::Phase {
                    player,
                    phase: Phase::Prepare,
                    begin: true
                } | EventData::TurnEndWindow {
                    player,
                    after: false
                } if player == owner
            );
            ok_phase && !eligible_bronze_tags(state, ctx.catalog, owner).is_empty()
        }
        "gwjinli_jiu" => matches!(
            *event,
            EventData::TurnEndWindow {
                player,
                after: false
            } if player == owner
        ),
        "gwlangshi" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.settlement.committed_damage.is_none()
                || d.prevented
                || d.amount <= 0
                || d.source != Some(owner)
                || !state.player(d.target).alive
            {
                return false;
            }
            if flag(state, owner, "gw_langshi_in_effect") > 0 {
                return false;
            }
            let min_hp = state.player(d.target).hp;
            state
                .alive_players()
                .any(|p| p != owner && p != d.target && state.player(p).hp >= min_hp)
        }
        "gwweitu" => {
            matches!(event, EventData::CardsDiscarded { player, cards } if *player == owner && !cards.is_empty())
                && state.player(owner).hujia < 3
        }
        "gwfutian" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && d.settlement.committed_damage.is_none()
                    && !d.prevented
                    && d.amount > 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner => state
                .alive_players()
                .any(|p| p != owner && !state.player(p).hand.is_empty()),
            _ => false,
        },
        "g_gw_yewu" => matches!(
            *event,
            EventData::TurnEndWindow {
                player,
                after: true
            } if player == owner
                && state.player(owner).hand.iter().any(|&c| {
                    matches!(ctx.catalog.cards[c as usize].name, "gw_wuyao" | "gw_lang")
                })
        ),
        _ => false,
    }
}

pub(super) fn machine_group_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut m = HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    };
    match event {
        EventData::TurnBegin { .. } if matches!(skill.name.as_str(), "sqlongwu" | "sqlonghuo") => {
            m.def_key = "sqlongyin".to_string();
        }
        EventData::Damage(d) => {
            m.target = Some(d.target);
        }
        _ => {}
    }
    m
}

pub(super) fn activate_group_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    let state = ctx.state;
    let ok = match skill.name.as_str() {
        "sqlonghuo" | "gwliaotian" => {
            let hs = &state.player(owner).hand;
            if skill.name == "sqlonghuo" {
                hs.iter().any(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(state, ctx.catalog, owner, c)
                })
            } else {
                hs.len() >= 2 && {
                    let first_red = is_red_card(ctx.catalog, hs[0]);
                    flag(state, owner, "gw_use2_cnt:gwliaotian") < 2
                        && hs.iter().all(|&c| is_red_card(ctx.catalog, c) == first_red)
                        && hs.iter().any(|&c| {
                            !crate::kernel::is_gw_dudayuanshuai2_locked(
                                state,
                                ctx.catalog,
                                owner,
                                c,
                            )
                        })
                }
            }
        }
        "gwjinli" => {
            state.player(owner).hand.iter().any(|&c| {
                !crate::kernel::is_gw_dudayuanshuai2_locked(state, ctx.catalog, owner, c)
            }) && state
                .alive_players()
                .any(|p| !has_named_skill(state, ctx.catalog, p, "gwjinli_jiu"))
        }
        "gwhuanbi" => !discardable_hand_and_equips(state, ctx.catalog, owner).is_empty(),
        "gwminxiang" => state
            .player(owner)
            .hand
            .iter()
            .any(|&c| minxiang_card_eligible(state, ctx.catalog, owner, c)),
        "gwjingtian" => {
            flag(state, owner, "gwjingtian_marks") > 0
                && !has_named_skill(state, ctx.catalog, owner, "gwjingtian2")
                && (!state.deck.is_empty() || !state.discard.is_empty())
        }
        "gwjingshi" => state.alive_players().any(|p| !state.player(p).hand.is_empty()),
        "gwgouhun" => {
            state.player(owner).hand.iter().any(|&c| {
                !crate::kernel::is_gw_dudayuanshuai2_locked(state, ctx.catalog, owner, c)
            }) && state
                .alive_players()
                .any(|p| p != owner && !state.player(p).hand.is_empty())
        }
        "gwyewu" => {
            !discardable_hand(state, ctx.catalog, owner).is_empty()
                && enemies_of(state, ctx.catalog, owner)
                    .into_iter()
                    .any(|e| !discardable_hand_and_equips(state, ctx.catalog, e).is_empty())
        }
        _ => false,
    };
    ok.then(|| HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(super) fn step_group_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "sqlongyin" => step_sqlongyin(m.owner, ctx),
        "sqlongnu" => step_sqlongnu(m, ctx),
        "sqlonghuo" => step_sqlonghuo(m, ctx),
        "sqlongwu" => step_sqlongwu(m, ctx),
        "jielue" => step_jielue(m, ctx),
        "gwhuanshuang" => step_gwhuanshuang(m, ctx),
        "gwjinli" => step_gwjinli(m, ctx),
        "gwjinli_jiu" => step_gwjinli_jiu(m.owner, ctx),
        "gwliaotian" => step_gwliaotian(m, ctx),
        "gwhuanbi" => step_gwhuanbi(m, ctx),
        "gwminxiang" => step_gwminxiang(m, ctx),
        "gwlangshi" => step_gwlangshi(m, ctx),
        "gwjingtian" => step_gwjingtian(m.owner, ctx),
        "gwjingshi" => step_gwjingshi(m, ctx),
        "gwweitu" => {
            if ctx.core.player(m.owner).hujia < 3 {
                ctx.change_hujia(m.owner, 1);
            }
            Flow::Done
        }
        "gwgouhun" => step_gwgouhun(m, ctx),
        "gwfutian" => step_gwfutian(m, ctx),
        "gwyewu" => step_gwyewu(m, ctx),
        "g_gw_yewu" => step_g_gw_yewu(m.owner, ctx),
        _ => Flow::Done,
    }
}

fn step_sqlongyin(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    let turn_stamp = ctx.core.turn.round.saturating_mul(16) + u32::from(owner) + 1;
    if flag(ctx.core, owner, "gw_sqlongyin_done_turn") == turn_stamp {
        return Flow::Done;
    }
    set_flag(ctx.core, owner, "gw_sqlongyin_done_turn", turn_stamp);
    let forms = ["sqlongnu", "sqlonghuo", "sqlongwu"];
    let cur = forms
        .iter()
        .copied()
        .find(|&f| has_named_skill(ctx.core, ctx.catalog, owner, f))
        .unwrap_or("sqlongwu");
    let pool: Vec<&str> = forms.iter().copied().filter(|&f| f != cur).collect();
    if pool.len() == 2 {
        let pick = pool[ctx.core.rng.gen_range(2) as usize];
        if let Some(old_def) = ctx.catalog.skill_by_name(cur) {
            ctx.remove_skill(owner, old_def);
        }
        if let Some(new_def) = ctx.catalog.skill_by_name(pick) {
            ctx.insert_skill(
                (owner, new_def),
                SkillInstance {
                    owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
    }
    Flow::Done
}

fn step_sqlongnu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let mut pool = ctx.core.deck.clone();
            let mut picks = Vec::new();
            for _ in 0..3 {
                if pool.is_empty() {
                    break;
                }
                let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                picks.push(pool.remove(idx));
            }
            if picks.is_empty() {
                return Flow::Done;
            }
            m.cards = picks;
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_SQLONGNU_PICK,
                build_ask_group_a,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !m.cards.contains(&picked) || !ctx.core.deck.contains(&picked) {
                return Flow::Done;
            }
            let picked_name = clean_gwent_name(ctx.catalog.cards[picked as usize].name);
            let dup = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .any(|&c| clean_gwent_name(ctx.catalog.cards[c as usize].name) == picked_name);
            if dup {
                m.card = Some(picked);
                return wait_ask(
                    m,
                    ctx,
                    HostStage::TargetChoice,
                    ASK_SQLONGNU_DAMAGE,
                    build_ask_group_a,
                );
            }
            if let Some(pos) = ctx.core.deck.iter().position(|&c| c == picked) {
                ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).hand.push(picked);
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: vec![picked],
                });
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(picked) = m.card.take() else {
                return Flow::Done;
            };
            let pos = ctx.core.deck.iter().position(|&c| c == picked);
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                if ctx.core.player(t).alive {
                    if let Some(i) = pos {
                        ctx.core.deck.remove(i);
                        ctx.to_discard(picked);
                    }
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Fire,
                        None,
                    )));
                    return Flow::Done;
                }
            }
            if let Some(i) = pos {
                ctx.core.deck.remove(i);
                if ctx.core.player(owner).alive {
                    ctx.core.player_mut(owner).hand.push(picked);
                    ctx.emit(EventData::CardsDrawn {
                        player: owner,
                        cards: vec![picked],
                    });
                } else {
                    ctx.to_discard(picked);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_sqlonghuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let hs: Vec<CardId> = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, owner, c))
                .collect();
            if hs.is_empty() {
                return Flow::Done;
            }
            m.cards = hs.clone();
            for c in hs {
                ctx.discard_from_hand(owner, c);
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let count = m.cards.len() as u32;
            if count > 0 && ctx.core.player(owner).alive {
                ctx.start_draw(owner, count);
            }
            m.targets = enemies_of(ctx.core, ctx.catalog, owner);
            m.stage = HostStage::CompareTargetPick;
            Flow::Yield
        }
        HostStage::CompareTargetPick => {
            while !m.targets.is_empty() {
                let e = m.targets.remove(0);
                if !ctx.core.player(e).alive {
                    continue;
                }
                let matching: Vec<CardId> = ctx
                    .core
                    .player(e)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| {
                        !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, e, c)
                            && m.cards.iter().any(|&dc| {
                                clean_gwent_name(ctx.catalog.cards[dc as usize].name)
                                    == clean_gwent_name(ctx.catalog.cards[c as usize].name)
                            })
                    })
                    .collect();
                if !matching.is_empty() {
                    let idx = ctx.core.rng.gen_range(matching.len() as u32) as usize;
                    ctx.discard_from_hand(e, matching[idx]);
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_sqlongwu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let hs = &ctx.core.player(owner).hand;
            let mut max_n = 1u32;
            for &c in hs {
                let nm = clean_gwent_name(ctx.catalog.cards[c as usize].name);
                let cnt = hs
                    .iter()
                    .filter(|&&x| clean_gwent_name(ctx.catalog.cards[x as usize].name) == nm)
                    .count() as u32;
                max_n = max_n.max(cnt);
            }
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, max_n);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive
                || sqlongwu_playable_hand(ctx.core, ctx.catalog, owner).is_empty()
            {
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_SQLONGWU_USE_CARD,
                build_ask_group_a,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c)
                || !b5b_card_can_use(ctx.core, ctx.catalog, owner, c, true)
            {
                return Flow::Done;
            }
            m.card = Some(c);
            m.target = None;
            if b5b_card_targets(ctx.core, ctx.catalog, owner, c).is_some() {
                return wait_ask(
                    m,
                    ctx,
                    HostStage::TargetChoice,
                    ASK_SQLONGWU_USE_TARGET,
                    build_ask_group_a,
                );
            }
            ctx.take_hand_card(owner, c);
            ctx.note_cards_lost(owner, &[c]);
            emit_huoge_use(ctx, owner, c, Vec::new());
            Flow::Done
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(c) = m.card.take() else {
                return Flow::Done;
            };
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if targets.is_empty() || !ctx.core.player(owner).hand.contains(&c) {
                return Flow::Done;
            }
            if ctx.catalog.card_kind(c) == CardKind::Jiedao {
                let holder = targets[0];
                if b5b_jiedao_victim_options(ctx.core, ctx.catalog, owner, holder).is_empty() {
                    return Flow::Done;
                }
                m.card = Some(c);
                m.target = Some(holder);
                return wait_ask(
                    m,
                    ctx,
                    HostStage::CardsAsk,
                    ASK_SQLONGWU_USE_TARGET,
                    build_ask_group_a,
                );
            }
            ctx.take_hand_card(owner, c);
            ctx.note_cards_lost(owner, &[c]);
            emit_huoge_use(ctx, owner, c, targets);
            Flow::Done
        }
        HostStage::CardsAsk => {
            clear_ask_id(ctx.core, owner);
            let (Some(c), Some(holder)) = (m.card.take(), m.target.take()) else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c) {
                return Flow::Done;
            }
            ctx.take_hand_card(owner, c);
            ctx.note_cards_lost(owner, &[c]);
            emit_huoge_use(ctx, owner, c, vec![holder, victim]);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_jielue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let mut by_friend: Vec<(PlayerId, Vec<CardId>)> = Vec::new();
            for f in friends_of(ctx.core, ctx.catalog, owner) {
                let usable: Vec<CardId> = ctx
                    .core
                    .player(f)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| b5b_card_can_use(ctx.core, ctx.catalog, owner, c, false))
                    .collect();
                if !usable.is_empty() {
                    by_friend.push((f, usable));
                }
            }
            let mut to_use = Vec::new();
            let mut providers = Vec::new();
            for _ in 0..2 {
                if by_friend.is_empty() {
                    break;
                }
                let fi = ctx.core.rng.gen_range(by_friend.len() as u32) as usize;
                let (f, mut list) = by_friend.remove(fi);
                let ci = ctx.core.rng.gen_range(list.len() as u32) as usize;
                let card = list.remove(ci);
                if ctx.core.player(f).hand.contains(&card) {
                    ctx.take_hand_card(f, card);
                    ctx.note_cards_lost(f, &[card]);
                    off_court_put(ctx.core, owner, card);
                    ctx.emit(EventData::CardsLost {
                        player: f,
                        cards: vec![card],
                        origins: vec![crate::event::LostOrigin::Hand],
                    });
                    to_use.push(card);
                    providers.push(f);
                }
            }
            m.cards = to_use;
            m.targets = providers;
            m.stage = HostStage::TargetChoice;
            advance_jielue_queue(m, ctx)
        }
        HostStage::TargetChoice | HostStage::SelectCard => {
            if m.stage == HostStage::SelectCard {
                clear_ask_id(ctx.core, owner);
                let targets: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                if let Some(&c) = m.cards.first() {
                    if ctx.catalog.card_kind(c) == CardKind::Jiedao {
                        if let Some(&holder) = targets.first() {
                            if !b5b_jiedao_victim_options(ctx.core, ctx.catalog, owner, holder)
                                .is_empty()
                            {
                                m.target = Some(holder);
                                return wait_ask(
                                    m,
                                    ctx,
                                    HostStage::CompareTargetPick,
                                    ASK_JIELUE_TARGET,
                                    build_ask_group_a,
                                );
                            }
                        }
                    }
                    let c = m.cards.remove(0);
                    off_court_take(ctx.core, owner, c);
                    if !targets.is_empty() {
                        mark_sha_no_count_if_sha(ctx.core, ctx.catalog, owner, c);
                        emit_huoge_use(ctx, owner, c, targets);
                        m.stage = HostStage::TargetChoice;
                        return Flow::Yield;
                    } else {
                        ctx.to_discard(c);
                    }
                }
            }
            advance_jielue_queue(m, ctx)
        }
        HostStage::CompareTargetPick => {
            clear_ask_id(ctx.core, owner);
            let holder = m.target.take();
            if !m.cards.is_empty() {
                let c = m.cards.remove(0);
                off_court_take(ctx.core, owner, c);
                if let (Some(h), Some(ChoiceOption::Player(v))) =
                    (holder, ctx.answer().first().copied())
                {
                    emit_huoge_use(ctx, owner, c, vec![h, v]);
                    m.stage = HostStage::TargetChoice;
                    return Flow::Yield;
                }
                ctx.to_discard(c);
            }
            advance_jielue_queue(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn advance_jielue_queue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    while let Some(&c) = m.cards.first() {
        if !b5b_card_can_use(ctx.core, ctx.catalog, owner, c, false) {
            m.cards.remove(0);
            off_court_take(ctx.core, owner, c);
            ctx.to_discard(c);
            continue;
        }
        m.target = None;
        if jielue_card_targets(ctx.core, ctx.catalog, owner, c).is_some() {
            return wait_ask(m, ctx, HostStage::SelectCard, ASK_JIELUE_TARGET, build_ask_group_a);
        }
        m.cards.remove(0);
        off_court_take(ctx.core, owner, c);
        mark_sha_no_count_if_sha(ctx.core, ctx.catalog, owner, c);
        emit_huoge_use(ctx, owner, c, Vec::new());
        return Flow::Yield;
    }
    for f in std::mem::take(&mut m.targets).into_iter().rev() {
        if ctx.core.player(f).alive {
            ctx.start_draw(f, 1);
        }
    }
    Flow::Done
}

fn step_gwhuanshuang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let mut tags = eligible_bronze_tags(ctx.core, ctx.catalog, owner);
            if tags.is_empty() {
                return Flow::Done;
            }
            let mut chosen = Vec::new();
            while chosen.len() < 3 && !tags.is_empty() {
                let i = ctx.core.rng.gen_range(tags.len() as u32) as usize;
                chosen.push(tags.swap_remove(i));
            }
            set_flag(ctx.core, owner, "gw_huanshuang_n", chosen.len() as u32);
            for (i, opt) in chosen.iter().enumerate() {
                let ChoiceOption::Tag(t) = *opt else { continue };
                set_flag(ctx.core, owner, &format!("gw_huanshuang_opt:{i}"), u32::from(t) + 1);
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_GWHUANSHUANG_PICK,
                build_ask_group_a,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(&(_, kind)) = BRONZE_SPELL_NAMES.get(tag as usize) else {
                return Flow::Done;
            };
            if let Some(def) = ctx.catalog.skill_by_name("gwhuanshuang_disable") {
                ctx.add_skill_until_turn_end(owner, def);
            }
            m.card = Some(u32::from(tag));
            set_flag(ctx.core, owner, "gw_huanshuang_casts", 2);
            cast_huanshuang_step(m, ctx, kind)
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let idx = m.card.unwrap_or(0) as usize;
            let Some(&(_, kind)) = BRONZE_SPELL_NAMES.get(idx) else {
                return Flow::Done;
            };
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            let rem = flag(ctx.core, owner, "gw_huanshuang_casts").saturating_sub(1);
            set_flag(ctx.core, owner, "gw_huanshuang_casts", rem);
            if !targets.is_empty() {
                m.stage = if rem > 0 {
                    HostStage::DrawSettle
                } else {
                    HostStage::HuanrenPick
                };
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: owner,
                    targets,
                    nature: DamageNature::Normal,
                    use_as: Some(kind),
                    cancelled: false,
                }));
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            let idx = m.card.unwrap_or(0) as usize;
            let Some(&(_, kind)) = BRONZE_SPELL_NAMES.get(idx) else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            cast_huanshuang_step(m, ctx, kind)
        }
        HostStage::HuanrenPick => Flow::Done,
        _ => Flow::Done,
    }
}

fn cast_huanshuang_step(m: &mut HostMachine, ctx: &mut Ctx<'_>, kind: CardKind) -> Flow {
    let owner = m.owner;
    if matches!(kind, CardKind::Poxiao | CardKind::Wenyi | CardKind::Shanbengshu) {
        let rem = flag(ctx.core, owner, "gw_huanshuang_casts").saturating_sub(1);
        set_flag(ctx.core, owner, "gw_huanshuang_casts", rem);
        m.stage = if rem > 0 {
            HostStage::DrawSettle
        } else {
            HostStage::HuanrenPick
        };
        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
            card: None,
            player: owner,
            targets: Vec::new(),
            nature: DamageNature::Normal,
            use_as: Some(kind),
            cancelled: false,
        }));
        return Flow::Yield;
    }
    if eligible_bronze_targets(ctx.core, ctx.catalog, owner, kind).is_empty() {
        set_flag(ctx.core, owner, "gw_huanshuang_casts", 0);
        return Flow::Done;
    }
    wait_ask(
        m,
        ctx,
        HostStage::TargetChoice,
        ASK_GWHUANSHUANG_TARGET,
        build_ask_group_a,
    )
}


pub(super) fn refund_turn_limit(ctx: &mut Ctx<'_>, owner: PlayerId, skill_name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(skill_name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}
