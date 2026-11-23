//! Batch U05-6a (`shahai-hearth` Part 1): Batch A 20 技能 (`hshuanyu` .. `hstianqi_shali`)。

use super::u05_hearth::{
    alive_others, discard_card_from_zone, emit_hearth_virtual, equip_card_to_player,
    exact_base_name, flag, gain_card_to_hand, grant_skill, has_any_skill, is_delayed_trick_hearth,
    is_hearth_enemy, meta, nature_for_kind, pick_random_hearth_derivable, remove_skill, rng_pick,
    rng_pick_mut, set_flag, spawn_card_by_base,
};
use super::u05_hearth_asks::{
    collect_player_answers, effective_hearth_card, hearth_card_needs_target,
    is_tanmi_card_usable, set_active_ask, wait_hs, ASK_HS_ASYOUZHANG_TARGET,
    ASK_HS_ASYOUZHANG_USE, ASK_HS_HSHUANLING_DISCARD, ASK_HS_HSHUANYU_DISCOVER,
    ASK_HS_HUALING_SKILL, ASK_HS_HUALING_TARGET, ASK_HS_OLDHUANJUE_PICK,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_a(name: &str) -> Option<TriggerMeta> {
    Some(match name {
        "hshuanyu" => meta(EventKind::Damage, Timing::Normal, 0, true, Vec::new()),
        "wxuying" => meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::TurnPhaseAfter, EventKind::CardResponded],
        ),
        "buwendingyibian_lose" => meta(EventKind::TurnPhaseAfter, Timing::Normal, 0, false, Vec::new()),
        "yibian" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "tqchuanyue" => meta(EventKind::PhaseBegin, Timing::Normal, 10, false, Vec::new()),
        "oldhuanjue" => meta(EventKind::CardUsed, Timing::Normal, 2, true, Vec::new()),
        "hshuanling" => meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        ),
        "hsxingyi" => meta(
            EventKind::SkillAfter,
            Timing::Normal,
            0,
            false,
            vec![EventKind::SkillLogAfter, EventKind::SkillToggled],
        ),
        "asyouzhang" => meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()),
        "ylyuchu" => meta(
            EventKind::Recover,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin, EventKind::TurnBegin],
        ),
        "nsaiqi" => meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new()),
        "hsnitai" => meta(EventKind::PhaseBegin, Timing::Normal, -10, false, Vec::new()),
        "hstianqi" => meta(EventKind::PhaseBegin, Timing::Normal, 30, false, Vec::new()),
        "hstianqi_dalian" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "hstianqi_shali" => meta(EventKind::Recover, Timing::Normal, 0, false, Vec::new()),
        // `muyin`, `hsyanxin`, `nsbeiming` execute via dedicated hooks / `nsaiqi` follow-up
        _ => return None,
    })
}

pub(super) fn limits_a(name: &str) -> SkillLimits {
    match name {
        "hspuzhao" | "hstianqi" | "oldhuanjue" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(super) fn has_active_a(name: &str) -> bool {
    matches!(name, "hualing" | "hspuzhao" | "hstianqi")
}

pub(super) fn activate_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let pl = ctx.state.player(owner);
    Some(match skill.name.as_str() {
        "hualing" => {
            let ready = flag(ctx.state, owner, "hs_hualing_ready_round");
            ctx.state.turn.round >= ready && !alive_others(ctx.state, owner).is_empty()
        }
        "hspuzhao" => pl
            .hand
            .iter()
            .chain(pl.equips.iter())
            .any(|&c| ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Heart),
        "hstianqi" => {
            let has_slot = |slot| {
                pl.equips
                    .iter()
                    .any(|&e| ctx.catalog.card_kind(e).equip_slot() == Some(slot))
            };
            let equips_full = has_slot(crate::catalog::EquipSlot::Weapon)
                && has_slot(crate::catalog::EquipSlot::Armor)
                && has_slot(crate::catalog::EquipSlot::HorsePlus)
                && has_slot(crate::catalog::EquipSlot::HorseMinus);
            !pl.equips.is_empty() || (!equips_full && !pl.hand.is_empty())
        }
        _ => return None,
    })
}

pub(super) fn filter_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "hshuanyu" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0
        ),
        "wxuying" => match event {
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } => *player == owner,
            EventData::TurnEndWindow { after: true, .. } => true,
            EventData::CardResponded { player, .. } => *player == owner,
            _ => false,
        },
        "buwendingyibian_lose" => matches!(event, EventData::TurnEndWindow { after: true, .. }),
        "yibian" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Play,
                begin: true,
            } if *player == owner
                && !ctx.state.player(owner).hand.iter().any(|&c| {
                    exact_base_name(ctx.catalog.cards[c as usize].name) == "hsfashu_buwendingyibian"
                })
        ),
        "tqchuanyue" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
        ),
        "oldhuanjue" => match event {
            EventData::CardUsed(u) => {
                u.targets.len() == 1
                    && u.targets[0] == owner
                    && u.player != owner
                    && flag(ctx.state, owner, "hs_oldhuanjue_used_turn") == 0
            }
            _ => false,
        },
        "hshuanling" => match event {
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "hs_hshuanling_cnt") > 0
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())
            }
            EventData::CardUsed(u) => {
                if u.use_as.is_some()
                    || u.card.is_none()
                    || u.targets.len() != 1
                    || !is_hearth_enemy(ctx.state, ctx.catalog, owner, u.player)
                {
                    return false;
                }
                let kind = u.effective_kind(ctx.catalog);
                if kind.is_basic_kind()
                    || kind.equip_slot().is_some()
                    || is_delayed_trick_hearth(kind)
                {
                    return false;
                }
                if matches!(
                    kind,
                    CardKind::Duel
                        | CardKind::Dismantle
                        | CardKind::Snatch
                        | CardKind::Linghunzhihuo
                        | CardKind::Zhaomingdan
                        | CardKind::Zhiliaobo
                        | CardKind::Shijieshu
                ) {
                    return true;
                }
                if kind == CardKind::Wuzhong {
                    let raw = u
                        .card
                        .and_then(|c| ctx.catalog.cards.get(c as usize))
                        .map(|c| c.name)
                        .unwrap_or("");
                    let exact = exact_base_name(raw);
                    return super::u05_hearth_a_tail::is_hshuanling_named_trick(exact);
                }
                false
            }
            _ => false,
        },
        "hsxingyi" => match event {
            EventData::SkillInvocation(inv)
                if inv.phase == crate::event::InvocationPhase::After =>
            {
                let player = inv.owner;
                let sdef = inv.def;
                player != owner
                    && player == ctx.state.turn.current
                    && is_hearth_enemy(ctx.state, ctx.catalog, owner, player)
                    && !has_any_skill(ctx.state, ctx.catalog, owner, ctx.catalog.skill_name(sdef))
            }
            EventData::SkillLog {
                owner: player,
                def: sdef,
                begin: false,
            } => {
                *player != owner
                    && *player == ctx.state.turn.current
                    && is_hearth_enemy(ctx.state, ctx.catalog, owner, *player)
                    && !has_any_skill(ctx.state, ctx.catalog, owner, ctx.catalog.skill_name(*sdef))
            }
            EventData::SkillToggled {
                player,
                skill: sdef,
                enabled: true,
                temporary: false,
            } => {
                let sname = ctx.catalog.skill_name(*sdef);
                let is_stock = ctx
                    .state
                    .player(*player)
                    .general_slots
                    .iter()
                    .flatten()
                    .any(|&g| {
                        ctx.catalog
                            .general_meta(g)
                            .is_some_and(|(_, _, sks, ..)| sks.contains(&sname))
                    });
                *player != owner
                    && *player == ctx.state.turn.current
                    && is_hearth_enemy(ctx.state, ctx.catalog, owner, *player)
                    && is_stock
                    && ctx
                        .state
                        .skills
                        .get(&(*player, *sdef))
                        .is_some_and(|i| i.enabled)
                    && !has_any_skill(ctx.state, ctx.catalog, owner, sname)
            }
            _ => false,
        },
        "asyouzhang" => match event {
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner => {
                let hand = &ctx.state.player(owner).hand;
                let has_basic = hand.iter().any(|&c| ctx.catalog.card_kind(c).is_basic_kind());
                let has_trick = hand.iter().any(|&c| {
                    let k = ctx.catalog.card_kind(c);
                    !k.is_basic_kind() && k.equip_slot().is_none()
                });
                let has_equip = hand
                    .iter()
                    .any(|&c| ctx.catalog.card_kind(c).equip_slot().is_some());
                !has_basic || !has_trick || !has_equip
            }
            _ => false,
        },
        "ylyuchu" => match event {
            EventData::Recover { player, amount, .. } => {
                *player == owner && *amount > 0 && flag(ctx.state, owner, "hs_ylyuchu_charges") < 3
            }
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "hs_ylyuchu_charges") > 0,
            EventData::TurnBegin { player } => {
                *player != owner
                    && flag(ctx.state, owner, &format!("hs_ylyuchu_on:{player}")) > 0
                    && flag(ctx.state, owner, "hs_ylyuchu_charges") > 0
            }
            _ => false,
        },
        "nsaiqi" => matches!(event, EventData::CardUsed(u) if u.player == owner),
        "hsnitai" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Play,
                begin: true,
            } if *player == owner
        ),
        "hstianqi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner && flag(ctx.state, owner, "hs_hstianqi_mask") == 0b1111
        ),
        "hstianqi_dalian" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
        ),
        "hstianqi_shali" => matches!(
            event,
            EventData::Recover { player, amount, .. } if *player == owner && *amount > 0
        ),
        _ => false,
    }
}

pub(super) fn step_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "hshuanyu" => step_hshuanyu(m, ctx),
        "wxuying" => step_wxuying(m, ctx),
        "buwendingyibian_lose" => {
            let cards: Vec<CardId> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    exact_base_name(ctx.catalog.cards[c as usize].name) == "hsfashu_buwendingyibian"
                        && flag(ctx.core, m.owner, &format!("hs_buwendingyibian_destroy:{c}")) > 0
                })
                .collect();
            for c in cards {
                set_flag(ctx.core, m.owner, &format!("hs_buwendingyibian_destroy:{c}"), 0);
                ctx.take_hand_card(m.owner, c);
                super::u05_hearth::finish_table_card_hearth(ctx, c, true);
            }
            Flow::Done
        }
        "yibian" => {
            if let Some(c) = spawn_card_by_base(ctx, "hsfashu_buwendingyibian") {
                gain_card_to_hand(ctx, m.owner, c);
            }
            Flow::Done
        }
        "hualing" => step_hualing(m, ctx),
        "tqchuanyue" => step_tqchuanyue(m, ctx),
        "oldhuanjue" => step_oldhuanjue(m, ctx),
        "hshuanling" => step_hshuanling(m, ctx),
        "hsxingyi" => step_hsxingyi(m, ctx),
        "asyouzhang" => step_asyouzhang(m, ctx),
        "ylyuchu" => step_ylyuchu(m, ctx),
        "nsaiqi" | "hsnitai" | "hspuzhao" | "hstianqi" | "hstianqi_dalian"
        | "hstianqi_shali" => super::u05_hearth_a_tail::step_a_tail(m, ctx),
        _ => Flow::Done,
    }
}

fn step_hshuanyu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let offered = pick_random_hearth_derivable(ctx, 3);
            if offered.is_empty() {
                ctx.start_draw(m.owner, 1);
                return Flow::Done;
            }
            m.cards = offered;
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_HSHUANYU_DISCOVER)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                ctx.core.detached_cards.retain(|&(x, _)| x != picked);
                gain_card_to_hand(ctx, m.owner, picked);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_wxuying(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(EventData::CardResponded { player, card }) =
        ctx.res.stack.last().map(|f| f.data.clone())
    {
        if player == m.owner {
            let src_card = super::u05_hearth_hooks::wxuying_effective_src(
                ctx.core,
                ctx.catalog,
                player,
                card,
            );
            let used_kind = ctx
                .core
                .declared_kinds
                .get(&card)
                .copied()
                .unwrap_or_else(|| ctx.catalog.card_kind(card));
            super::u05_hearth_hooks::wxuying_apply_hide(ctx, player, card);
            super::u05_hearth_hooks::wxuying_transform_hand(ctx, player, used_kind, Some(src_card));
        }
        return Flow::Done;
    }
    let is_turn_after = ctx
        .res
        .stack
        .last()
        .is_some_and(|f| matches!(f.data, EventData::TurnEndWindow { after: true, .. }));
    if is_turn_after {
        // `wxuying_clear` (`character/hearth/skill.js:121-127`): clear transient `was_shadow` ONLY;
        // retain `hs_wxuying_shadow:{cid}` & `declared_kinds` for cards remaining in hand for off-turn responses.
        let hand_cards = &ctx.core.player(m.owner).hand;
        let stale: Vec<(CardId, bool)> = ctx
            .core
            .vm_aux
            .flags
            .keys()
            .filter(|(p, _)| *p == m.owner)
            .filter_map(|(_, k)| {
                if let Some(s) = k.strip_prefix("hs_wxuying_was_shadow:") {
                    s.parse::<CardId>().ok().map(|c| (c, false))
                } else if let Some(s) = k.strip_prefix("hs_wxuying_kind:") {
                    s.parse::<CardId>()
                        .ok()
                        .filter(|c| !hand_cards.contains(c))
                        .map(|c| (c, true))
                } else {
                    None
                }
            })
            .collect();
        for (cid, drop_kind) in stale {
            set_flag(ctx.core, m.owner, &format!("hs_wxuying_was_shadow:{cid}"), 0);
            if drop_kind {
                set_flag(ctx.core, m.owner, &format!("hs_wxuying_kind:{cid}"), 0);
                set_flag(ctx.core, m.owner, &format!("hs_wxuying_shadow:{cid}"), 0);
                set_flag(ctx.core, m.owner, &format!("hs_wxuying_src_card:{cid}"), 0);
                ctx.core.declared_kinds.remove(&cid);
            }
        }
        return Flow::Done;
    }
    let old: Vec<CardId> = ctx
        .core
        .player(m.owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            exact_base_name(ctx.catalog.cards[c as usize].name) == "hsfashu_anyingjingxiang"
                || flag(ctx.core, m.owner, &format!("hs_wxuying_shadow:{c}")) > 0
        })
        .collect();
    for c in old {
        ctx.core.declared_kinds.remove(&c);
        set_flag(ctx.core, m.owner, &format!("hs_wxuying_kind:{c}"), 0);
        set_flag(ctx.core, m.owner, &format!("hs_wxuying_shadow:{c}"), 0);
        set_flag(ctx.core, m.owner, &format!("hs_wxuying_src_card:{c}"), 0);
        ctx.core.player_mut(m.owner).hand.retain(|&x| x != c);
        super::u05_hearth::finish_table_card_hearth(ctx, c, true);
    }
    if let Some(fresh) = spawn_card_by_base(ctx, "hsfashu_anyingjingxiang") {
        gain_card_to_hand(ctx, m.owner, fresh);
    }
    Flow::Done
}

fn step_hualing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_HUALING_TARGET),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            set_flag(ctx.core, m.owner, "hs_hualing_ready_round", ctx.core.turn.round + 3);
            let candidate_defs: Vec<SkillDefId> = ctx
                .core
                .skills
                .keys()
                .filter(|&&(p, def)| {
                    p == t
                        && !has_any_skill(
                            ctx.core,
                            ctx.catalog,
                            m.owner,
                            ctx.catalog.skill_name(def),
                        )
                })
                .map(|&(_, def)| def)
                .collect();
            if candidate_defs.is_empty() {
                hualing_reinit_target(ctx, t);
                return Flow::Done;
            }
            set_flag(ctx.core, m.owner, "hs_choice_count", candidate_defs.len() as u32);
            for (i, &def) in candidate_defs.iter().enumerate() {
                set_flag(ctx.core, m.owner, &format!("hs_choice_sk:{i}"), u32::from(def) + 1);
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_HUALING_SKILL)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() {
                let raw = flag(ctx.core, m.owner, &format!("hs_choice_sk:{idx}"));
                if raw > 0 {
                    let old_def = flag(ctx.core, m.owner, "hs_hualing_granted_def");
                    if old_def > 0 {
                        let old_name = ctx.catalog.skill_name((old_def - 1) as u16).to_string();
                        remove_skill(ctx, m.owner, &old_name);
                    }
                    let new_name = ctx.catalog.skill_name((raw - 1) as u16).to_string();
                    grant_skill(ctx, m.owner, &new_name, false);
                    set_flag(ctx.core, m.owner, "hs_hualing_granted_def", raw);
                }
            }
            if let Some(t) = m.target {
                hualing_reinit_target(ctx, t);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn hualing_reinit_target(ctx: &mut Ctx<'_>, t: PlayerId) {
    let cur = ctx.core.player(t).general_slots[0];
    let pool: Vec<u16> = (0..ctx.catalog.general_count() as u16)
        .filter(|&g| Some(g) != cur)
        .collect();
    if let Some(next_g) = rng_pick(ctx, &pool) {
        let old_hp = ctx.core.player(t).hp;
        let removed: Vec<SkillDefId> = ctx
            .core
            .skills
            .keys()
            .filter(|(p, _)| *p == t)
            .map(|(_, d)| *d)
            .collect();
        for d in removed {
            ctx.core.skills.remove(&(t, d));
        }
        ctx.core.player_mut(t).general_slots[0] = Some(next_g);
        crate::state::apply_general_assembly(ctx.core, ctx.catalog, t, 0);
        let mhp = ctx.core.player(t).max_hp as i32;
        ctx.core.player_mut(t).hp = old_hp.min(mhp);
        ctx.log(LogEvent::SkillToggled {
            player: t,
            skill: "hualing_reinit",
            enabled: true,
        });
    }
}

fn step_tqchuanyue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    const POOL: [&str; 12] = [
        "oldhanshuang",
        "oldyanshu",
        "oldfbeifa",
        "oldmalymowang",
        "oldfenlie",
        "oldshifa",
        "oldfushi",
        "oldhuanjue",
        "oldzhanhou",
        "oldduxin",
        "oldenze",
        "oldyulu",
    ];
    let avail: Vec<&str> = POOL
        .iter()
        .copied()
        .filter(|&sk| {
            ctx.catalog.skill_by_name(sk).is_some()
                && !has_any_skill(ctx.core, ctx.catalog, m.owner, sk)
        })
        .collect();
    if let Some(pick) = rng_pick(ctx, &avail) {
        let old = flag(ctx.core, m.owner, "hs_tqchuanyue_def");
        if old > 0 {
            let old_name = ctx.catalog.skill_name((old - 1) as u16).to_string();
            remove_skill(ctx, m.owner, &old_name);
        }
        if let Some(def) = ctx.catalog.skill_by_name(pick) {
            grant_skill(ctx, m.owner, pick, false);
            set_flag(ctx.core, m.owner, "hs_tqchuanyue_def", u32::from(def) + 1);
        }
    }
    Flow::Done
}

fn step_oldhuanjue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let kinds = [
                CardKind::Sha,
                CardKind::Duel,
                CardKind::Dismantle,
                CardKind::Snatch,
                CardKind::Tao,
                CardKind::Wuzhong,
                CardKind::Le,
            ];
            let mut choices = Vec::new();
            let mut idx_pool: Vec<usize> = (0..kinds.len()).collect();
            while choices.len() < 3 && !idx_pool.is_empty() {
                let k_idx = rng_pick_mut(ctx, &mut idx_pool).unwrap_or(0);
                let target_kind = kinds[k_idx];
                if let Some(c) = ctx
                    .core
                    .detached_cards
                    .iter()
                    .map(|&(cid, _)| cid)
                    .find(|&cid| {
                        let def = &ctx.catalog.cards[cid as usize];
                        def.kind == target_kind
                            && !matches!(exact_base_name(def.name), "du" | "shihuawuqi")
                            && !choices.contains(&cid)
                    })
                {
                    choices.push(c);
                }
            }
            if choices.is_empty() {
                return Flow::Done;
            }
            set_flag(ctx.core, m.owner, "hs_oldhuanjue_used_turn", 1);
            // Store offered cards in `hs_oldhuanjue_offer` (keep them in detached until picked!)
            m.cards = choices;
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_OLDHUANJUE_PICK)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                ctx.core.detached_cards.retain(|&(x, _)| x != picked);
                let new_kind = ctx.catalog.card_kind(picked);
                let fid = ctx.current_frame_id();
                if let Some(frame) = ctx.res.frame_mut(fid) {
                    let u = frame.card_use_mut();
                    let old_c = u.card.replace(picked);
                    u.use_as = Some(new_kind);
                    ctx.core.table.push(picked);
                    if let Some(oc) = old_c {
                        ctx.core.table.retain(|&x| x != oc);
                        ctx.to_discard(oc);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn hshuanling_pick_target(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    kind: CardKind,
    cid_plus: u32,
) -> Option<PlayerId> {
    if cid_plus == 0 {
        return None;
    }
    let orig_cid = (cid_plus - 1) as CardId;
    let exact = exact_base_name(ctx.catalog.cards[orig_cid as usize].name);
    let beneficial = super::u05_hearth_a_tail::is_hshuanling_beneficial(kind, exact);
    let legal: Vec<PlayerId> =
        super::u05_hearth_asks::single_target_options(ctx.core, ctx.catalog, owner, orig_cid)
            .into_iter()
            .filter_map(|o| match o {
                ChoiceOption::Player(p) => Some(p),
                _ => None,
            })
            .collect();
    let preferred: Vec<PlayerId> = legal
        .iter()
        .copied()
        .filter(|&p| {
            if beneficial {
                p == owner || super::u05_hearth::is_hearth_friend(ctx.core, ctx.catalog, owner, p)
            } else {
                is_hearth_enemy(ctx.core, ctx.catalog, owner, p)
            }
        })
        .collect();
    rng_pick(ctx, &preferred)
}

fn step_hshuanling(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let in_card_use = ctx
        .res
        .stack
        .last()
        .is_some_and(|f| matches!(f.data, EventData::CardUsed(_)));
    if in_card_use {
        if let Some(cid) = m.card {
            let kind = ctx.catalog.card_kind(cid);
            let code = crate::archive::kind_code(kind) as u32;
            let cnt = flag(ctx.core, m.owner, "hs_hshuanling_cnt");
            set_flag(ctx.core, m.owner, &format!("hs_hshuanling_eff_{cnt}"), code);
            set_flag(
                ctx.core,
                m.owner,
                &format!("hs_hshuanling_cid_{cnt}"),
                u32::from(cid) + 1,
            );
            set_flag(ctx.core, m.owner, "hs_hshuanling_cnt", cnt + 1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            let cnt = flag(ctx.core, m.owner, "hs_hshuanling_cnt");
            if cnt == 0 {
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_HSHUANLING_DISCARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let discarded: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if discarded.is_empty() {
                return Flow::Done;
            }
            for &dc in &discarded {
                discard_card_from_zone(ctx, m.owner, dc);
            }
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: discarded.clone(),
            });
            m.card = Some(discarded.len() as u32);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(m.owner, discarded.len() as u32);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let left = m.card.unwrap_or(0);
            if left == 0 || !ctx.core.player(m.owner).alive {
                return Flow::Done;
            }
            let total = flag(ctx.core, m.owner, "hs_hshuanling_cnt");
            for idx in 0..total {
                let code = flag(ctx.core, m.owner, &format!("hs_hshuanling_eff_{idx}"));
                let cid_plus = flag(ctx.core, m.owner, &format!("hs_hshuanling_cid_{idx}"));
                let Ok(kind) = crate::archive::kind_from_code(code as u16) else {
                    continue;
                };
                let Some(t) = hshuanling_pick_target(ctx, m.owner, kind, cid_plus) else {
                    continue;
                };
                for i in (idx + 1)..total {
                    let v = flag(ctx.core, m.owner, &format!("hs_hshuanling_eff_{i}"));
                    let c = flag(ctx.core, m.owner, &format!("hs_hshuanling_cid_{i}"));
                    set_flag(ctx.core, m.owner, &format!("hs_hshuanling_eff_{}", i - 1), v);
                    set_flag(ctx.core, m.owner, &format!("hs_hshuanling_cid_{}", i - 1), c);
                }
                set_flag(ctx.core, m.owner, &format!("hs_hshuanling_eff_{}", total - 1), 0);
                set_flag(ctx.core, m.owner, &format!("hs_hshuanling_cid_{}", total - 1), 0);
                set_flag(ctx.core, m.owner, "hs_hshuanling_cnt", total - 1);
                m.card = Some(left - 1);
                if kind == CardKind::Wuzhong && cid_plus > 0 {
                    set_flag(ctx.core, m.owner, "hs_virtual_card_src", cid_plus);
                }
                emit_hearth_virtual(ctx, m.owner, kind, vec![t]);
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hsxingyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let used_def = ctx
        .res
        .stack
        .iter()
        .rev()
        .find_map(|f| match &f.data {
            EventData::SkillInvocation(inv)
                if inv.phase == crate::event::InvocationPhase::After =>
            {
                Some(inv.def)
            }
            EventData::SkillLog {
                def,
                begin: false,
                ..
            } => Some(*def),
            EventData::SkillToggled {
                skill,
                enabled: true,
                temporary: false,
                ..
            } => Some(*skill),
            _ => None,
        });
    if let Some(sdef) = used_def {
        let sname = ctx.catalog.skill_name(sdef).to_string();
        if !has_any_skill(ctx.core, ctx.catalog, m.owner, &sname) {
            grant_skill(ctx, m.owner, &sname, true);
        }
    }
    Flow::Done
}

fn step_asyouzhang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    loop {
        match m.stage {
            HostStage::Act => {
                // Phase step idx stored in `hs_asyou_step`: 0=basic, 1=trick, 2=equip
                let step = flag(ctx.core, m.owner, "hs_asyou_step");
                if step >= 3 {
                    set_flag(ctx.core, m.owner, "hs_asyou_step", 0);
                    return Flow::Done;
                }
                let hand = &ctx.core.player(m.owner).hand;
                let has_cat = match step {
                    0 => hand.iter().any(|&c| ctx.catalog.card_kind(c).is_basic_kind()),
                    1 => hand.iter().any(|&c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none()
                    }),
                    _ => hand
                        .iter()
                        .any(|&c| ctx.catalog.card_kind(c).equip_slot().is_some()),
                };
                set_flag(ctx.core, m.owner, "hs_asyou_step", step + 1);
                if has_cat {
                    continue;
                }
                let found_idx = ctx.core.deck.iter().rposition(|&c| {
                    let k = ctx.catalog.card_kind(c);
                    match step {
                        0 => k.is_basic_kind(),
                        1 => !k.is_basic_kind() && k.equip_slot().is_none(),
                        _ => k.equip_slot().is_some(),
                    }
                });
                let Some(idx) = found_idx else {
                    continue;
                };
                let gained = ctx.core.deck.remove(idx);
                gain_card_to_hand(ctx, m.owner, gained);
                if is_tanmi_card_usable(ctx.core, ctx.catalog, m.owner, gained) {
                    m.cards = vec![gained];
                    return wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_ASYOUZHANG_USE);
                }
            }
            HostStage::SelectCard => {
                set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
                let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                    m.stage = HostStage::Act;
                    continue;
                };
                if hearth_card_needs_target(ctx.core, ctx.catalog, m.owner, picked) {
                    return wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_ASYOUZHANG_TARGET);
                }
                use_gained_card(ctx, m.owner, picked, Vec::new());
                m.stage = HostStage::Act;
                return Flow::Yield;
            }
            HostStage::TargetChoice => {
                set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
                let picked_targets = collect_player_answers(ctx.answer());
                if let Some(&picked) = m.cards.first() {
                    if !picked_targets.is_empty() {
                        use_gained_card(ctx, m.owner, picked, picked_targets);
                        m.stage = HostStage::Act;
                        return Flow::Yield;
                    }
                }
                m.stage = HostStage::Act;
            }
            _ => return Flow::Done,
        }
    }
}

pub(super) fn use_gained_card(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    card: CardId,
    mut targets: Vec<PlayerId>,
) {
    let (_, kind) = effective_hearth_card(ctx.core, ctx.catalog, owner, card);
    if kind.is_equip() {
        ctx.take_hand_card(owner, card);
        equip_card_to_player(ctx, owner, card);
    } else {
        if kind == CardKind::Jiedao && targets.len() == 1 {
            let holder = targets[0];
            let Some(victim) = ctx.core.alive_players().find(|&v| {
                v != owner
                    && v != holder
                    && ctx.core.computed_distance(ctx.catalog, holder, v)
                        <= ctx.core.attack_range(ctx.catalog, holder)
            }) else {
                return;
            };
            targets.push(victim);
        }
        if targets.is_empty() {
            targets = match kind {
                CardKind::SavageAttack
                | CardKind::ArcheryAttack
                | CardKind::Yuansuhuimie => alive_others(ctx.core, owner),
                CardKind::Taoyuan | CardKind::Wugu => ctx.core.seat_order_from(owner),
                CardKind::Jiu => Vec::new(),
                _ => vec![owner],
            };
        }
        let use_as = ctx
            .core
            .declared_kinds
            .get(&card)
            .copied()
            .filter(|&dk| dk != ctx.catalog.card_kind(card));
        ctx.take_hand_card(owner, card);
        ctx.core.table.push(card);
        ctx.log(LogEvent::CardPlayed {
            card,
            player: owner,
            targets: targets.clone(),
            presentation: None,
        });
        ctx.emit(EventData::CardUsed(crate::machine::card_use_with_nature(
            card,
            owner,
            targets,
            nature_for_kind(kind),
            use_as,
        )));
    }
}

fn step_ylyuchu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cur_ev = ctx.res.stack.last().map(|f| &f.data);
    match cur_ev {
        Some(EventData::Recover { amount, .. }) => {
            let cur = flag(ctx.core, m.owner, "hs_ylyuchu_charges");
            let next = (cur + *amount).min(3);
            set_flag(ctx.core, m.owner, "hs_ylyuchu_charges", next);
        }
        Some(EventData::Phase {
            phase: crate::event::Phase::End,
            ..
        }) => {
            let seats = ctx.core.players.len() as PlayerId;
            for p in 0..seats {
                set_flag(ctx.core, m.owner, &format!("hs_ylyuchu_on:{p}"), 0);
            }
            let charges = flag(ctx.core, m.owner, "hs_ylyuchu_charges") as usize;
            let mut others = alive_others(ctx.core, m.owner);
            for _ in 0..charges {
                if let Some(p) = rng_pick_mut(ctx, &mut others) {
                    set_flag(ctx.core, m.owner, &format!("hs_ylyuchu_on:{p}"), 1);
                }
            }
        }
        Some(EventData::TurnBegin { player }) => {
            let p = *player;
            set_flag(ctx.core, m.owner, &format!("hs_ylyuchu_on:{p}"), 0);
            let cur = flag(ctx.core, m.owner, "hs_ylyuchu_charges");
            if cur > 0 {
                set_flag(ctx.core, m.owner, "hs_ylyuchu_charges", cur - 1);
                set_flag(ctx.core, m.owner, "extra_turn_insert_queued", 1);
                set_flag(ctx.core, m.owner, "hs_ylyuchu_extra_draw_minus", 1);
                ctx.start_draw(m.owner, 2);
            }
        }
        _ => {}
    }
    Flow::Done
}
