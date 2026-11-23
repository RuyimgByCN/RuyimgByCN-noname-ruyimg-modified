//! U05-7d 杀海拾遗 swd 人物技能战斗/伤害/觉醒修正（`swd_u05_7d_fixes_combat.rs`）。

use super::super::*;
use super::swd_u05_7d::{discard_owned_card_once, refund_once_used, virtual_sha_safe};
use super::swd_u05_7d_fixes::{he_cards, is_enemy_of};
use std::collections::BTreeSet;

pub(super) fn step_swd_u05_7d_combat(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::SwdXuehuang => match m.stage {
            HostStage::Act => {
                ctx.core.player_mut(m.owner).xuehuang_used = true;
                let cards: Vec<CardId> = ctx.core.player(m.owner).hand.iter().copied().collect();
                if cards.is_empty() {
                    return Flow::Done;
                }
                let num = cards.len() as u32;
                for c in cards {
                    ctx.discard_from_hand(m.owner, c);
                }
                m.card = Some(num);
                m.stage = HostStage::ShuchuiTargetLoop;
                continue_xuehuang_shot(m, ctx)
            }
            HostStage::ShuchuiTargetLoop => continue_xuehuang_shot(m, ctx),
            _ => Flow::Done,
        },
        HostBehavior::Moyan => match m.stage {
            HostStage::Act => {
                let reds: Vec<ChoiceOption> = ctx
                    .core
                    .player(m.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(
                            ctx.catalog.effective_card_suit(ctx.core, m.owner, *c),
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                let others = ctx.core.alive_players().filter(|p| *p != m.owner).count();
                let hp_gap = (ctx.core.player(m.owner).max_hp as i32
                    - ctx.core.player(m.owner).hp)
                    .max(0) as usize;
                let max = reds.len().min(others).min(hp_gap);
                if max == 0 {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "moyan_cards",
                        options: reds,
                        min: 1,
                        max,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != m.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                let need = picked.len();
                if options.len() < need {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.cards = picked;
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "moyan_targets",
                        options,
                        min: need,
                        max: need,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let targets: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if targets.len() != m.cards.len() || targets.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let cost = std::mem::take(&mut m.cards);
                for c in cost {
                    discard_owned_card_once(ctx, m.owner, c);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "魔焰",
                    owner: m.owner,
                    presentation: None,
                });
                m.targets = targets;
                m.stage = HostStage::ShuchuiTargetLoop;
                continue_moyan_loop(m, ctx)
            }
            HostStage::ShuchuiTargetLoop => continue_moyan_loop(m, ctx),
            _ => Flow::Done,
        },
        HostBehavior::SwdHuajian => match m.stage {
            HostStage::Act => {
                let pool = he_cards(ctx.core, m.owner);
                if pool.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "huajian_discard",
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
                if !he_cards(ctx.core, m.owner).contains(&picked) {
                    return Flow::Done;
                }
                m.card = Some(picked);
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        engine_native_kernel::kernel::can_target_sha_no_range(
                            ctx.core,
                            ctx.catalog,
                            m.owner,
                            *p,
                            CardKind::Sha,
                        )
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "huajian_target",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some(picked) = m.card.take() else {
                    return Flow::Done;
                };
                if !engine_native_kernel::kernel::can_target_sha_no_range(
                    ctx.core,
                    ctx.catalog,
                    m.owner,
                    t,
                    CardKind::Sha,
                ) {
                    return Flow::Done;
                }
                if !he_cards(ctx.core, m.owner).contains(&picked) {
                    return Flow::Done;
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, "vm_va_sha_no_count".to_string()), 1);
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: Some(picked),
                    player: m.owner,
                    targets: vec![t],
                    nature: crate::event::DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::Lxianglong => match m.stage {
            HostStage::Act => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != m.owner
                            && !ctx.catalog.has_undist(ctx.core, *p)
                            && ctx.core.in_attack_range(ctx.catalog, m.owner, *p)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "lxianglong_target",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "李香龙",
                    owner: m.owner,
                    presentation: None,
                });
                virtual_sha_safe(
                    ctx,
                    m.owner,
                    CardKind::Sha,
                    crate::event::DamageNature::Normal,
                    vec![t],
                );
                m.stage = HostStage::ShuchuiTargetLoop;
                Flow::Yield
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdRexue => match m.stage {
            HostStage::Act => {
                let Some(actor) = m.target.filter(|&a| ctx.core.player(a).alive) else {
                    return Flow::Done;
                };
                let options =
                    engine_native_kernel::kernel::usable_sha_options(
                        ctx.core,
                        ctx.catalog,
                        m.owner,
                        actor,
                    );
                if options.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "rexue_sha",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let Some(actor) = m.target.filter(|&a| ctx.core.player(a).alive) else {
                    return Flow::Done;
                };
                let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some((use_as, is_mutou)) = engine_native_kernel::kernel::sha_card_use_as(
                    ctx.core,
                    ctx.catalog,
                    m.owner,
                    card,
                ) else {
                    return Flow::Done;
                };
                let eff_kind = use_as.unwrap_or_else(|| ctx.catalog.card_kind(card));
                if !engine_native_kernel::kernel::can_target_sha_no_range(
                    ctx.core,
                    ctx.catalog,
                    m.owner,
                    actor,
                    eff_kind,
                ) {
                    return Flow::Done;
                }
                if is_mutou {
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::host::GJ_MUTOU_SKILL_KEY)
                    {
                        if let Some(inst) = ctx.core.skills.get_mut(&(m.owner, def)) {
                            inst.once_used = true;
                        }
                    }
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, "vm_va_sha_no_count".to_string()), 1);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "热血",
                    owner: m.owner,
                    presentation: None,
                });
                ctx.emit(EventData::CardUsed(super::sha_card_use_data(
                    ctx,
                    m.owner,
                    card,
                    vec![actor],
                    use_as,
                )));
                ctx.start_draw(m.owner, 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::Kunlunjing => {
            let begin = matches!(
                ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data),
                Some(EventData::TurnBegin { .. })
            );
            if !begin {
                let mut snap = Vec::new();
                for p in 0..ctx.core.players.len() as PlayerId {
                    let pl = ctx.core.player(p);
                    snap.push((p, pl.hand.clone(), pl.equips.clone(), pl.judge.clone()));
                }
                let hp = ctx.core.player(m.owner).hp;
                ctx.core.turn.kunlunjing_snapshot = snap;
                ctx.core.turn.kunlunjing_hp = hp;
                return Flow::Done;
            }
            if m.stage == HostStage::Act {
                m.stage = HostStage::SelectNature;
                return Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec::confirm("trigger_confirm"),
                };
            }
            if !ctx.answer().contains(&ChoiceOption::Confirm) {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "昆仑镜",
                owner: m.owner,
                presentation: None,
            });
            let snap = std::mem::take(&mut ctx.core.turn.kunlunjing_snapshot);
            let snap_judges: BTreeSet<CardId> = snap
                .iter()
                .flat_map(|(_, _, _, j)| j.iter().copied())
                .collect();
            let alive: Vec<PlayerId> = ctx.core.alive_players().collect();
            for p in alive {
                let hand: Vec<CardId> = ctx.core.player(p).hand.iter().copied().collect();
                for c in hand {
                    ctx.discard_from_hand(p, c);
                }
                let equips: Vec<CardId> = ctx.core.player(p).equips.iter().copied().collect();
                for c in equips {
                    discard_owned_card_once(ctx, p, c);
                }
                let judges: Vec<CardId> = ctx.core.player(p).judge.iter().copied().collect();
                for c in judges {
                    ctx.core.player_mut(p).judge.retain(|x| *x != c);
                    if !snap_judges.contains(&c) {
                        ctx.core.declared_kinds.remove(&c);
                    }
                    ctx.to_discard(c);
                    ctx.note_cards_lost(p, &[c]);
                    ctx.emit(EventData::CardsLost {
                        player: p,
                        cards: vec![c],
                        origins: vec![crate::event::LostOrigin::Judge],
                    });
                    ctx.log(LogEvent::CardDiscarded {
                        player: p,
                        cards: vec![c],
                    });
                }
            }
            for (p, hand, equips, judge) in snap {
                if !ctx.core.player(p).alive {
                    continue;
                }
                for c in hand {
                    kunlunjing_move_to(ctx, c, p, 0);
                }
                for c in equips {
                    kunlunjing_move_to(ctx, c, p, 1);
                }
                for c in judge {
                    kunlunjing_move_to(ctx, c, p, 2);
                }
            }
            Flow::Done
        }
        HostBehavior::SwdDaixing3 => {
            let dmg_fid = ctx
                .res
                .stack
                .iter()
                .rev()
                .find(|f| matches!(f.data, EventData::Damage(_)))
                .map(|f| f.id);
            let absorbed = dmg_fid
                .and_then(|fid| {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(m.owner, format!("damage_hujia_absorbed:{fid}")))
                })
                .unwrap_or(0);
            if absorbed > 0 {
                let cur = ctx.core.player(m.owner).daixing_count;
                ctx.core.player_mut(m.owner).daixing_count =
                    cur.saturating_sub(absorbed.min(u32::from(u8::MAX)) as u8);
            }
            Flow::Done
        }
        HostBehavior::SwdYinguo => match m.stage {
            HostStage::Act => {
                let pool = he_cards(ctx.core, m.owner);
                if pool.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "yinguo_discard",
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
                let Some(dmg_fid) = ctx
                    .res
                    .stack
                    .iter()
                    .rev()
                    .find(|f| matches!(f.data, EventData::Damage(_)))
                    .map(|f| f.id)
                else {
                    return Flow::Done;
                };
                if !discard_owned_card_once(ctx, m.owner, picked) {
                    return Flow::Done;
                }
                let frame = ctx.res.frame_mut(dmg_fid).expect("damage frame");
                if let EventData::Damage(d) = &mut frame.data {
                    let t = d.target;
                    d.target = d.source.expect("filter guarantees source");
                    d.source = Some(t);
                }
                if let Some(y2_def) = ctx.catalog.skill_by_behavior(HostBehavior::SwdYinguo2) {
                    ctx.insert_skill(
                        (m.owner, y2_def),
                        crate::state::SkillInstance {
                            owner: m.owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, format!("yinguo2_frame:{dmg_fid}")), 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdSusheng => match m.stage {
            HostStage::Act => {
                let Some(p) = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .map(|f| &f.data)
                    .and_then(|data| match data {
                        EventData::Death { player, .. } => Some(*player),
                        _ => None,
                    })
                else {
                    return Flow::Done;
                };
                m.target = Some(p);
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(m.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "susheng_discard",
                        options,
                        min: 0,
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
                if !ctx.core.player(m.owner).hand.contains(&picked) {
                    return Flow::Done;
                }
                ctx.discard_from_hand(m.owner, picked);
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                {
                    let pl = ctx.core.player_mut(t);
                    pl.hp = 1;
                    if pl.max_hp < 1 {
                        pl.max_hp = 1;
                    }
                    pl.death_cancelled = true;
                }
                if let Some(s2_def) = ctx.catalog.skill_by_behavior(HostBehavior::SwdSusheng2) {
                    ctx.insert_skill(
                        (m.owner, s2_def),
                        crate::state::SkillInstance {
                            owner: m.owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn continue_xuehuang_shot(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let rem = m.card.unwrap_or(0);
    if rem == 0 || !ctx.core.player(m.owner).alive {
        m.card = None;
        return Flow::Done;
    }
    let mut enemies: Vec<PlayerId> = ctx
        .core
        .alive_players()
        .filter(|&p| is_enemy_of(ctx.core, m.owner, p) && !ctx.catalog.has_undist(ctx.core, p))
        .collect();
    if enemies.is_empty() {
        m.card = None;
        return Flow::Done;
    }
    let mut picked = Vec::new();
    for _ in 0..2 {
        if enemies.is_empty() {
            break;
        }
        let j = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
        picked.push(enemies.swap_remove(j));
    }
    m.card = Some(rem - 1);
    virtual_sha_safe(
        ctx,
        m.owner,
        CardKind::FireSha,
        crate::event::DamageNature::Fire,
        picked,
    );
    if rem - 1 == 0 {
        Flow::Done
    } else {
        Flow::Yield
    }
}

fn continue_moyan_loop(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while !m.targets.is_empty() {
        let t = m.targets.remove(0);
        if ctx.core.player(t).alive {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(m.owner),
                t,
                1,
                crate::event::DamageNature::Fire,
                None,
            )));
            if !m.targets.is_empty() {
                return Flow::Yield;
            }
        }
    }
    Flow::Done
}
