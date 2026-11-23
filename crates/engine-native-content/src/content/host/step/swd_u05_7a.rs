//! U05 `杀海拾遗 · swd` Cards + Card Skills Half 1 (`U05-7a`: tags `1280..=1287`)。
//!
//! 承载：
//! - `U05SwdDujianCard` (`1280`): 毒箭手牌比色（同色弃己牌并令目标失 1 体力；异色暂记 `dujian2`）。
//! - `U05SwdPangufu` (`1281`): 盘古斧武器技（造成伤害后令受伤者自选弃 1 张 `he` 牌）。
//! - `U05SwdHufuViewAs` (`1282`): `g_hufu_sha` / `g_hufu_shan` / `g_hufu_jiu`（主动出杀/饮酒 + 响应面转化）。
//! - `U05SwdYuchanEquip` (`1283`): `g_yuchan_equip`（出牌限一次弃 ≥1 基本手牌摸等量）。
//! - `U05SwdZhiluxiaohuDraw` (`1284`): `zhiluxiaohu`（指路小虎虚拟杀造成伤害后摸 1）。
//! - `U05SwdHuanpodanGuard` (`1285`): `huanpodan_skill`（濒死防死弃全牌至 1 血摸 1 移技）。
//! - `U05SwdForgingEnd` (`1286`): 六件器族锻造回合结束触发（`*_equip1..5` 共 30 技）。
//! - `U05SwdPassiveMarker` (`1287`): 固有修饰/窗口标记（`qiankundai` / `zhufangshenshi` 等正式包绑定）。

use super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask as build_ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(in crate::content::host) fn trigger_meta(
    behavior: HostBehavior,
    name: &str,
) -> Option<TriggerMeta> {
    let meta = |event: EventKind, priority: i32| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match behavior {
        HostBehavior::U05SwdPangufu => Some(meta(EventKind::DamageEnd, 55)),
        HostBehavior::U05SwdZhiluxiaohuDraw => Some(meta(EventKind::DamageEnd, 0)),
        HostBehavior::U05SwdHuanpodanGuard => Some(meta(EventKind::Death, 0)),
        HostBehavior::U05SwdForgingEnd if !name.is_empty() => {
            Some(meta(EventKind::TurnPhaseEnd, 0))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn limits(behavior: HostBehavior, name: &str) -> SkillLimits {
    match behavior {
        HostBehavior::U05SwdYuchanEquip => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        HostBehavior::U05SwdForgingEnd if name.starts_with("qinglongzhigui_equip") => {
            SkillLimits::default()
        }
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn has_phase_use_activation(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    match behavior {
        HostBehavior::U05SwdYuchanEquip => name == "g_yuchan_equip",
        HostBehavior::U05SwdHufuViewAs => matches!(name, "g_hufu_sha" | "g_hufu_jiu"),
        _ => false,
    }
}

pub(super) fn is_red_card(catalog: &Catalog, core: &GameStateCore, owner: PlayerId, c: CardId) -> bool {
    matches!(
        catalog.effective_card_suit(core, owner, c),
        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
    )
}

pub(super) fn hand_or_equip(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand.iter().chain(pl.equips.iter()).copied().collect()
}

pub(super) fn has_forged_yuchan_skill(catalog: &Catalog, core: &GameStateCore, owner: PlayerId) -> bool {
    core.skills.iter().any(|((seat, def), inst)| {
        *seat == owner
            && inst.enabled
            && catalog.skills[*def as usize].name.starts_with("yuchan")
            && catalog.skills[*def as usize].name.contains("_equip")
    }) || core.player(owner).equips.iter().any(|c| {
        catalog.card_kind(*c).yuchan_auto_view_as().is_some()
            || core
                .player(owner)
                .card_mods
                .get(c)
                .is_some_and(|m| catalog.card_kind(*m).yuchan_auto_view_as().is_some())
    })
}

pub(in crate::content::host) fn activate(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    match skill.behavior {
        HostBehavior::U05SwdYuchanEquip => {
            if !has_forged_yuchan_skill(ctx.catalog, ctx.state, owner) {
                return None;
            }
            let has_basic = ctx
                .state
                .player(owner)
                .hand
                .iter()
                .any(|c| ctx.catalog.card_kind(*c).is_basic_kind());
            if !has_basic {
                return None;
            }
            Some(HostMachine {
                def_key: skill.name.clone(),
                behavior: skill.behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::SelectCard,
            })
        }
        HostBehavior::U05SwdHufuViewAs => match skill.name.as_str() {
            "g_hufu_sha" => {
                if super::swd_u05_7a_hufu::hufu_sha_eligible_cards(ctx.catalog, ctx.state, owner)
                    .is_empty()
                {
                    return None;
                }
                Some(HostMachine {
                    def_key: skill.name.clone(),
                    behavior: skill.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                })
            }
            "g_hufu_jiu" => {
                if super::swd_u05_7a_hufu::hufu_jiu_eligible_cards(ctx.catalog, ctx.state, owner)
                    .is_empty()
                {
                    return None;
                }
                Some(HostMachine {
                    def_key: skill.name.clone(),
                    behavior: skill.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                })
            }
            _ => None,
        },
        _ => None,
    }
}

pub(in crate::content::host) fn filter_with_frame(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    _frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05SwdPangufu => matches!(
            event,
            EventData::Damage(d)
            if d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && !d.settlement.notrigger
                && ctx.state.player(d.target).alive
                && !hand_or_equip(ctx.state, d.target).is_empty()
        ),
        HostBehavior::U05SwdZhiluxiaohuDraw => matches!(
            event,
            EventData::Damage(d)
            if d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && !d.settlement.notrigger
                && d.card.is_none_or(|c| ctx.catalog.card_kind(c).is_sha_like())
        ),
        HostBehavior::U05SwdHuanpodanGuard => matches!(
            event,
            EventData::Death { player, .. } if *player == owner
        ),
        HostBehavior::U05SwdForgingEnd => match event {
            EventData::TurnEndWindow { player, after: false } if *player == owner => {
                super::swd_u05_7a_forging::forging_matches(skill.name.as_str(), ctx, owner)
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let target = match event {
        EventData::Damage(d) => Some(d.target),
        EventData::Death { player, .. } => Some(*player),
        _ => None,
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::U05SwdDujianCard => step_dujian(m, ctx),
        HostBehavior::U05SwdPangufu => step_pangufu(m, ctx),
        HostBehavior::U05SwdHufuViewAs => super::swd_u05_7a_hufu::step_hufu_view_as(m, ctx),
        HostBehavior::U05SwdYuchanEquip => step_yuchan_equip(m, ctx),
        HostBehavior::U05SwdZhiluxiaohuDraw => {
            ctx.start_draw(m.owner, 1);
            Flow::Done
        }
        HostBehavior::U05SwdHuanpodanGuard => step_huanpodan(m, ctx),
        HostBehavior::U05SwdForgingEnd => super::swd_u05_7a_forging::step_forging_end(m, ctx),
        HostBehavior::U05SwdPassiveMarker => step_swd_7a_card_effect(m, ctx),
        _ => Flow::Done,
    }
}

fn step_swd_7a_card_effect(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "zhufangshenshi_card_effect" => {
            if let Some(t) = m.target {
                ctx.core.turn.zhufangshenshi_target = Some(t);
            }
            if let Some(c) = m.card {
                ctx.core.table.retain(|x| *x != c);
                ctx.to_discard(c);
            }
        }
        "shencaojie_card_effect" => {
            if let Some(c) = m.card {
                let key = (m.owner, format!("swd_scj_dmg:{c}"));
                if let Some(dmg_fid) = ctx.core.vm_aux.flags.remove(&key) {
                    if let Some(frame) = ctx.res.frame_mut(dmg_fid) {
                        if let EventData::Damage(d2) = &mut frame.data {
                            if !d2.prevented {
                                if d2.target == m.owner {
                                    d2.amount = d2.amount.saturating_sub(1);
                                } else {
                                    d2.amount += 1;
                                }
                            }
                        }
                    }
                }
                ctx.core.table.retain(|x| *x != c);
                ctx.to_discard(c);
            }
        }
        _ => {}
    }
    Flow::Done
}

pub(in crate::content::host) fn card_intrinsic_swd_7a(
    ctx: &mut Ctx<'_>,
    u: &crate::event::CardUseData,
    card: CardId,
    effective_kind: CardKind,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    match effective_kind {
        CardKind::Dujian => Some(Box::new(HostMachine {
            def_key: "dujian".to_string(),
            behavior: HostBehavior::U05SwdDujianCard,
            owner: u.player,
            target: u.targets.first().copied(),
            card: Some(card),
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        })),
        CardKind::Guisheqi => Some(Box::new(HostMachine {
            def_key: "guisheqi_card".to_string(),
            behavior: HostBehavior::GuisheqiCard,
            owner: u.player,
            target: Some(*u.targets.first().expect("guisheqi target")),
            card: Some(card),
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        })),
        CardKind::Xianluhui => Some(Box::new(HostMachine {
            def_key: "xianluhui_card".to_string(),
            behavior: HostBehavior::XianluhuiCard,
            owner: u.player,
            target: None,
            card: Some(card),
            cards: Vec::new(),
            targets: u.targets.clone(),
            stage: HostStage::Act,
        })),
        CardKind::Zhufangshenshi | CardKind::Shencaojie => {
            let def_key = if effective_kind == CardKind::Zhufangshenshi {
                "zhufangshenshi_card_effect"
            } else {
                "shencaojie_card_effect"
            };
            let effect = Box::new(HostMachine {
                def_key: def_key.to_string(),
                behavior: HostBehavior::U05SwdPassiveMarker,
                owner: u.player,
                target: u.targets.first().copied(),
                card: Some(card),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            });
            Some(Box::new(crate::kernel::wrap_nullify_window(
                ctx, card, u.player, effect,
            )))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn host_card_use_after_swd(
    ctx: &mut Ctx<'_>,
    u: &crate::event::CardUseData,
) {
    let eff = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
    if eff == CardKind::Shencaojie {
        if let Some(c) = u.card {
            ctx.core
                .vm_aux
                .flags
                .remove(&(u.player, format!("swd_scj_dmg:{c}")));
        }
    }
    if !u.cancelled
        && eff == CardKind::Zhufangshenshi
        && ctx.core.player(u.player).alive
    {
        ctx.start_draw(u.player, 1);
    }
    super::swd_equipment_skills::try_push_shenmiguo_after_use(ctx, u);
    super::swd_equipment_skills::try_push_yuchan_swap_after_use(ctx, u);
}

fn step_dujian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cleanup = |m: &mut HostMachine, ctx: &mut Ctx<'_>| {
        if let Some(c) = m.card {
            ctx.to_discard(c);
            ctx.core.table.retain(|x| *x != c);
            m.card = None;
        }
    };
    match m.stage {
        HostStage::Act => {
            let Some(t) = m.target else {
                cleanup(m, ctx);
                return Flow::Done;
            };
            if !ctx.core.player(t).alive
                || ctx.core.player(t).hand.is_empty()
                || ctx.core.player(m.owner).hand.is_empty()
            {
                cleanup(m, ctx);
                return Flow::Done;
            }
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "u05_dujian_own_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            let Some(card1) = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            }) else {
                cleanup(m, ctx);
                return Flow::Done;
            };
            let Some(t) = m.target else {
                cleanup(m, ctx);
                return Flow::Done;
            };
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                cleanup(m, ctx);
                return Flow::Done;
            }
            m.cards = vec![card1];
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            m.stage = HostStage::TargetChoice;
            Flow::WaitChoice {
                observer: t,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "u05_dujian_target_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::TargetChoice => {
            let Some(card2) = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            }) else {
                cleanup(m, ctx);
                return Flow::Done;
            };
            let Some(card1) = m.cards.first().copied() else {
                cleanup(m, ctx);
                return Flow::Done;
            };
            let Some(t) = m.target else {
                cleanup(m, ctx);
                return Flow::Done;
            };
            let red1 = is_red_card(ctx.catalog, ctx.core, m.owner, card1);
            let red2 = is_red_card(ctx.catalog, ctx.core, t, card2);
            if red1 == red2 {
                if ctx.core.player(m.owner).hand.contains(&card1) {
                    ctx.discard_from_hand(m.owner, card1);
                }
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
                }
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((t, "u05_dujian2_turn".to_string()), 1);
            }
            cleanup(m, ctx);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_pangufu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            let opts: Vec<ChoiceOption> = hand_or_equip(ctx.core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: t,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "u05_pangufu_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                discard_he_card(ctx, t, c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yuchan_equip(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::SelectCard => {
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .copied()
                .filter(|c| ctx.catalog.card_kind(*c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                refund_host_once_used(ctx, m.owner, &m.def_key);
                return Flow::Done;
            }
            let max = opts.len();
            m.stage = HostStage::Act;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "u05_yuchan_equip_discard",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            }
        }
        HostStage::Act => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                refund_host_once_used(ctx, m.owner, &m.def_key);
                return Flow::Done;
            }
            let mut discarded = Vec::new();
            for c in picked {
                if ctx.core.player(m.owner).hand.contains(&c) {
                    ctx.discard_from_hand(m.owner, c);
                    discarded.push(c);
                }
            }
            if !discarded.is_empty() {
                ctx.start_draw(m.owner, discarded.len() as u32);
                ctx.emit(EventData::CardsDiscarded {
                    player: m.owner,
                    cards: discarded,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_huanpodan(_m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = _m.owner;
    ctx.core.player_mut(owner).death_cancelled = true;
    let he = hand_or_equip(ctx.core, owner);
    for c in he {
        discard_he_card(ctx, owner, c);
    }
    if let Some(def) = ctx.catalog.skill_by_name("huanpodan_skill") {
        ctx.core.skills.remove(&(owner, def));
    }
    if ctx.core.player(owner).max_hp < 1 {
        ctx.core.player_mut(owner).max_hp = 1;
    }
    ctx.core.player_mut(owner).hp = 1;
    ctx.start_draw(owner, 1);
    Flow::Done
}

pub(super) fn discard_he_card(ctx: &mut Ctx<'_>, owner: PlayerId, card: CardId) {
    if ctx.core.player(owner).hand.contains(&card) {
        ctx.discard_from_hand(owner, card);
    } else if let Some(pos) = ctx.core.player(owner).equips.iter().position(|x| *x == card) {
        let old = ctx.core.player_mut(owner).equips.remove(pos);
        ctx.core.declared_slots.remove(&old);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, old);
        ctx.to_discard(old);
        ctx.log(LogEvent::CardDiscarded {
            player: owner,
            cards: vec![old],
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let cards_ask = |obs: PlayerId, prompt: &'static str, opts: Vec<CardId>, min: usize, max: usize, allow_cancel: bool| {
        Some(build_ask(
            obs,
            prompt,
            ChoiceKind::SelectCards,
            opts.into_iter().map(ChoiceOption::Card).collect(),
            min,
            max,
            allow_cancel,
        ))
    };
    match (behavior, stage) {
        (HostBehavior::U05SwdDujianCard, HostStage::SelectCard) => {
            cards_ask(owner, "u05_dujian_own_card", core.player(owner).hand.clone(), 1, 1, false)
        }
        (HostBehavior::U05SwdDujianCard, HostStage::TargetChoice) => {
            let t = target?;
            cards_ask(t, "u05_dujian_target_card", core.player(t).hand.clone(), 1, 1, false)
        }
        (HostBehavior::U05SwdPangufu, HostStage::SelectCard) => {
            let t = target?;
            cards_ask(t, "u05_pangufu_discard", hand_or_equip(core, t), 1, 1, false)
        }
        (HostBehavior::U05SwdHufuViewAs, _) => {
            super::swd_u05_7a_hufu::ask_hufu(stage, owner, cards, core, catalog)
        }
        (HostBehavior::U05SwdYuchanEquip, HostStage::Act) => {
            let basics: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.card_kind(*c).is_basic_kind())
                .collect();
            let max = basics.len();
            cards_ask(owner, "u05_yuchan_equip_discard", basics, 1, max, true)
        }
        (HostBehavior::U05SwdForgingEnd, _) => {
            super::swd_u05_7a_forging::ask_forging(stage, owner, core, catalog)
        }
        _ => None,
    }
}
