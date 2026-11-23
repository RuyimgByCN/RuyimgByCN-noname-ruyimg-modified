//! U05 `杀海拾遗 · swd` Cards + Card Skills Half 2 (`U05-7b`: tags `1310..=1339`)。
//!
//! 承载：
//! - 基本牌效果：`U05SwdXiangyuyeCard` (`1310`)、`U05SwdHuanpodanCard` (`1311`)
//! - 装备/衍生技：`kongxin` (`1312`)、`kongxin2` (`1313`)、`qinglianxindeng` (`1314`)、
//!   `hslingjian_*` 家族 (`1315..=1323`)、`jiguanyaoshu_skill` (`1324`)、
//!   `lingjianduanzao` (`1325`)、`_lingjianduanzao` (`1326`)、`qinglonglingzhu` (`1327`)、
//!   `xingjunyan` (`1328`)、`baihupifeng` (`1329`)、`fengxueren` (`1330`)、`chilongya` (`1331`)、
//!   `chilongya2` (`1332`)、`chilongfengxue` (`1333`)、`old_longfan` (`1334`)、`xixue` (`1335`)、
//!   `guangshatianyi` (`1336`)、`sadengjinhuan` (`1337`)、`guiyanfadao` (`1338`)、`tianxianjiu` (`1339`)。

#[path = "swd_u05_7b_asks.rs"]
pub(super) mod asks;
#[path = "swd_u05_7b_hooks.rs"]
mod hooks;
#[path = "swd_u05_7b_lingjian.rs"]
pub(in crate::content::host) mod lingjian;

pub(crate) use hooks::{
    host_card_use_after_u05_7b, host_draw_begin_step_u05_7b, host_sha_end_step_u05_7b,
    host_sha_hit_step_u05_7b, on_outer_settled_u05_7b, on_phase_end_u05_7b,
};

use super::swd_u05_7a::{discard_he_card, hand_or_equip};
use super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(in crate::content::host) fn trigger_meta(
    behavior: HostBehavior,
    name: &str,
) -> Option<TriggerMeta> {
    let meta = |event: EventKind, priority: i32, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match behavior {
        HostBehavior::U05SwdKongxin2 => Some(meta(EventKind::Dying, 10, Vec::new())),
        HostBehavior::U05SwdQinglianxindeng => Some(meta(EventKind::DamageBefore, 15, Vec::new())),
        HostBehavior::U05SwdLingjianDmgSrc
        | HostBehavior::U05SwdLingjianDmgTgt
        | HostBehavior::U05SwdQinglonglingzhu
        | HostBehavior::U05SwdXixue
        | HostBehavior::U05SwdTianxianjiu => Some(meta(EventKind::DamageEnd, 0, Vec::new())),
        HostBehavior::U05SwdLingjianLoseTemp => {
            Some(meta(EventKind::CardsLost, 0, vec![EventKind::CardsDiscarded]))
        }
        HostBehavior::U05SwdLingjianEquipped => Some(meta(EventKind::Equipped, 0, Vec::new())),
        HostBehavior::U05SwdLingjianDrawPhase => Some(meta(EventKind::DrawPhaseBegin, 0, Vec::new())),
        HostBehavior::U05SwdLingjianPhaseEnd => {
            if name == "hslingjian_xingtigaizao" {
                Some(meta(EventKind::PhaseEnd, 0, Vec::new()))
            } else {
                Some(meta(EventKind::TurnPhaseEnd, 0, Vec::new()))
            }
        }
        HostBehavior::U05SwdLingjianShaBegin => Some(meta(EventKind::ShaBegin, 0, Vec::new())),
        HostBehavior::U05SwdJiguanyaoshuSkill => {
            Some(meta(EventKind::CardsLost, 0, Vec::new()))
        }
        HostBehavior::U05SwdXingjunyan
        | HostBehavior::U05SwdChilongya
        | HostBehavior::U05SwdChilongya2
        | HostBehavior::U05SwdGuangshatianyi => Some(meta(EventKind::DamageBegin, 0, Vec::new())),
        HostBehavior::U05SwdBaihupifeng => Some(meta(EventKind::PhaseEnd, 0, Vec::new())),
        HostBehavior::U05SwdFengxueren => Some(meta(EventKind::DamageBefore, 0, Vec::new())),
        HostBehavior::U05SwdSadengjinhuan => Some(meta(EventKind::ShaDodged, 0, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits(behavior: HostBehavior, _name: &str) -> SkillLimits {
    match behavior {
        HostBehavior::U05SwdKongxin
        | HostBehavior::U05SwdLingjianPhaseUse
        | HostBehavior::U05SwdOldLongfan => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn has_phase_use_activation(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    match behavior {
        HostBehavior::U05SwdKongxin => name == "kongxin",
        HostBehavior::U05SwdLingjianPhaseUse => lingjian::is_lingjian_equip5_active(name),
        HostBehavior::U05SwdForgeAction => name == "_lingjianduanzao",
        HostBehavior::U05SwdOldLongfan => name == "old_longfan",
        _ => false,
    }
}

pub(in crate::content::host) fn activate(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    if !ctx.state.player(owner).alive {
        return None;
    }
    let mk = || HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    };
    match skill.behavior {
        HostBehavior::U05SwdKongxin => {
            let own_h = !ctx.state.player(owner).hand.is_empty();
            let has_other = ctx
                .state
                .alive_players()
                .any(|p| p != owner && !ctx.state.player(p).hand.is_empty());
            (own_h && has_other).then(mk)
        }
        HostBehavior::U05SwdLingjianPhaseUse => {
            lingjian::can_activate_equip5(skill, ctx, owner).then(mk)
        }
        HostBehavior::U05SwdForgeAction => {
            lingjian::can_activate_duanzao(ctx, owner).then(mk)
        }
        HostBehavior::U05SwdOldLongfan => ctx
            .state
            .alive_players()
            .next()
            .is_some()
            .then(mk),
        _ => None,
    }
}

pub(in crate::content::host) fn machine(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let target = match event {
        EventData::Damage(d) => Some(d.target),
        EventData::Dying { player, .. } => Some(*player),
        EventData::ShaDodged { target, .. } => Some(*target),
        _ => None,
    };
    let card = match event {
        EventData::Damage(d) => d.card,
        EventData::Equipped { card, .. } => Some(*card),
        EventData::ShaDodged { card, .. } => *card,
        _ => None,
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(super) fn has_equip_kind(core: &GameStateCore, catalog: &Catalog, p: PlayerId, kind: CardKind) -> bool {
    core.player(p)
        .equips
        .iter()
        .any(|c| catalog.card_kind(*c) == kind)
}

fn is_sha_card(catalog: &Catalog, card: Option<CardId>) -> bool {
    card.is_some_and(|c| catalog.card_kind(c).is_sha_like())
}

fn armor_ignored_for_damage(
    core: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
) -> bool {
    source.is_some_and(|src| {
        crate::rvm_bridge::modifiers::d34_l28_mods::source_ignores_armor(
            core, catalog, src, target, card, None, nature,
        )
    })
}

pub(in crate::content::host) fn filter_with_frame(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    _frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.behavior {
        HostBehavior::U05SwdKongxin2 => {
            matches!(event, EventData::Dying { player, .. } if *player == owner)
        }
        HostBehavior::U05SwdQinglianxindeng => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_trick_kind())
                    && !armor_ignored_for_damage(ctx.state, ctx.catalog, d.source, owner, d.card, d.nature)
        ),
        HostBehavior::U05SwdLingjianDmgSrc
        | HostBehavior::U05SwdLingjianDmgTgt
        | HostBehavior::U05SwdLingjianLoseTemp
        | HostBehavior::U05SwdLingjianEquipped
        | HostBehavior::U05SwdLingjianDrawPhase
        | HostBehavior::U05SwdLingjianPhaseEnd
        | HostBehavior::U05SwdLingjianShaBegin
        | HostBehavior::U05SwdJiguanyaoshuSkill => lingjian::filter(skill, ctx, owner, event),
        HostBehavior::U05SwdQinglonglingzhu => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && matches!(d.nature, DamageNature::Fire | DamageNature::Thunder)
                    && ctx.state.player(d.target).alive
                    && !hand_or_equip(ctx.state, d.target).is_empty()
                    && !has_equip_kind(ctx.state, ctx.catalog, owner, CardKind::Qinglonglingzhu)
        ),
        HostBehavior::U05SwdXingjunyan => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && (d.source == Some(owner) || d.target == owner)
                    && is_sha_card(ctx.catalog, d.card)
                    && !has_equip_kind(ctx.state, ctx.catalog, owner, CardKind::Xingjunyan)
        ),
        HostBehavior::U05SwdBaihupifeng => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } if *player == owner
                && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                && ctx.state.alive_players().all(|p| ctx.state.player(owner).hp <= ctx.state.player(p).hp)
        ),
        HostBehavior::U05SwdFengxueren => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && !d.from_chain
                    && is_sha_card(ctx.catalog, d.card)
                    && ctx.state.player(d.target).alive
                    && !ctx.state.player(d.target).face_down
                    && !has_equip_kind(ctx.state, ctx.catalog, owner, CardKind::Fengxueren)
        ),
        HostBehavior::U05SwdChilongya => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && !d.from_chain
                    && d.nature == DamageNature::Fire
                    && !has_equip_kind(ctx.state, ctx.catalog, owner, CardKind::Chilongya)
        ),
        HostBehavior::U05SwdChilongya2 => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && is_sha_card(ctx.catalog, d.card)
        ),
        HostBehavior::U05SwdXixue => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && is_sha_card(ctx.catalog, d.card)
                    && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                    && !has_equip_kind(ctx.state, ctx.catalog, owner, CardKind::Xixueguizhihuan)
        ),
        HostBehavior::U05SwdGuangshatianyi => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && !armor_ignored_for_damage(ctx.state, ctx.catalog, d.source, owner, d.card, d.nature)
        ),
        HostBehavior::U05SwdSadengjinhuan => matches!(
            event,
            EventData::ShaDodged { source, target, .. }
                if *source == owner
                    && ctx.state.player(*target).alive
                    && !has_equip_kind(ctx.state, ctx.catalog, owner, CardKind::Sadengjinhuan)
                    && ctx
                        .state
                        .vm_aux
                        .flags
                        .get(&(owner, format!("u05_sadeng_handled:{target}")))
                        .copied()
                        .unwrap_or(0)
                        == 0
        ),
        HostBehavior::U05SwdTianxianjiu => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && is_sha_card(ctx.catalog, d.card)
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::U05SwdXiangyuyeCard => step_xiangyuye(m, ctx),
        HostBehavior::U05SwdHuanpodanCard => step_huanpodan_card(m, ctx),
        HostBehavior::U05SwdKongxin => step_kongxin(m, ctx),
        HostBehavior::U05SwdKongxin2 => {
            if let Some(def) = ctx.catalog.skill_by_name("kongxin2") {
                ctx.core.skills.remove(&(m.owner, def));
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(m.owner, "u05_kongxin2_swap".to_string()));
            Flow::Done
        }
        HostBehavior::U05SwdQinglianxindeng => {
            ctx.current_damage_mut().prevented = true;
            ctx.log(LogEvent::DamagePrevented {
                target: m.owner,
                presentation: None,
            });
            Flow::Done
        }
        HostBehavior::U05SwdLingjianDmgSrc
        | HostBehavior::U05SwdLingjianDmgTgt
        | HostBehavior::U05SwdLingjianLoseTemp
        | HostBehavior::U05SwdLingjianEquipped
        | HostBehavior::U05SwdLingjianDrawPhase
        | HostBehavior::U05SwdLingjianPhaseEnd
        | HostBehavior::U05SwdLingjianPhaseUse
        | HostBehavior::U05SwdLingjianShaBegin
        | HostBehavior::U05SwdLingjianPassive
        | HostBehavior::U05SwdJiguanyaoshuSkill
        | HostBehavior::U05SwdLingjianduanzao
        | HostBehavior::U05SwdForgeAction => lingjian::step(m, ctx),
        HostBehavior::U05SwdQinglonglingzhu => step_qinglonglingzhu(m, ctx),
        HostBehavior::U05SwdXingjunyan | HostBehavior::U05SwdChilongya => {
            ctx.current_damage_mut().amount += 1;
            Flow::Done
        }
        HostBehavior::U05SwdBaihupifeng => step_baihupifeng(m, ctx),
        HostBehavior::U05SwdFengxueren => step_fengxueren(m, ctx),
        HostBehavior::U05SwdChilongya2 => step_chilongya2(m, ctx),
        HostBehavior::U05SwdChilongfengxue => Flow::Done,
        HostBehavior::U05SwdOldLongfan => step_old_longfan(m, ctx),
        HostBehavior::U05SwdXixue => {
            let fid = ctx.current_event_frame_id();
            let amt = match ctx.res.frame(fid).map(|f| &f.data) {
                Some(EventData::Damage(d)) => d.amount.max(0) as u32,
                _ => 0,
            };
            if amt > 0 {
                ctx.recover(m.owner, amt, None);
            }
            Flow::Done
        }
        HostBehavior::U05SwdGuangshatianyi => {
            if ctx.core.rng.gen_range(3) == 0 {
                let d = ctx.current_damage_mut();
                if d.amount > 0 {
                    d.amount -= 1;
                }
            }
            Flow::Done
        }
        HostBehavior::U05SwdSadengjinhuan => hooks::step_sadengjinhuan(m, ctx),
        HostBehavior::U05SwdGuiyanfadao => hooks::step_guiyanfadao(m, ctx),
        HostBehavior::U05SwdTianxianjiu => {
            ctx.start_draw(m.owner, 2);
            if let Some(def) = ctx.catalog.skill_by_name("tianxianjiu") {
                ctx.core.skills.remove(&(m.owner, def));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xiangyuye(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        discard_body_card(m, ctx);
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            discard_body_card(m, ctx);
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            let black = black_hand_cards(ctx.core, ctx.catalog, t);
            if black.is_empty() {
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
                return Flow::Done;
            }
            m.stage = HostStage::SelectCard;
            asks::wait_select_cards(t, "u05_xiangyuye_discard", black, 1, 1, true)
        }
        HostStage::SelectCard => {
            let pick = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            if let Some(c) = pick.filter(|c| {
                ctx.core.player(t).hand.contains(c)
                    && matches!(
                        ctx.catalog.effective_card_suit(ctx.core, t, *c),
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
            }) {
                ctx.discard_from_hand(t, c);
            } else if ctx.core.player(t).alive {
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_huanpodan_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    discard_body_card(m, ctx);
    if let Some(t) = m.target {
        if ctx.core.player(t).alive {
            if let Some(def) = ctx.catalog.skill_by_name("huanpodan_skill") {
                ctx.ensure_skill(
                    (t, def),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }
    }
    Flow::Done
}

fn step_kongxin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let opts: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| p != owner && !ctx.core.player(p).hand.is_empty())
                .collect();
            if opts.is_empty() || ctx.core.player(owner).hand.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::TargetChoice;
            asks::wait_select_players(owner, "u05_kongxin_target", opts, true)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                refund_host_once_used(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            if t == owner || !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            m.target = Some(t);
            let hand = ctx.core.player(owner).hand.clone();
            m.stage = HostStage::ComparePlayerPick;
            asks::wait_select_cards(owner, "u05_kongxin_pindian_own", hand, 1, 1, false)
        }
        HostStage::ComparePlayerPick => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c1) {
                return Flow::Done;
            }
            m.cards = vec![c1];
            let thand = ctx.core.player(t).hand.clone();
            if thand.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::CompareTargetPick;
            asks::wait_select_cards(t, "u05_kongxin_pindian_target", thand, 1, 1, false)
        }
        HostStage::CompareTargetPick => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(c1) = m.cards.first().copied() else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if ctx.core.player(owner).hand.contains(&c1) {
                ctx.discard_from_hand(owner, c1);
            }
            if ctx.core.player(t).hand.contains(&c2) {
                ctx.discard_from_hand(t, c2);
            }
            let r1 = ctx.catalog.cards[c1 as usize].rank;
            let r2 = ctx.catalog.cards[c2 as usize].rank;
            if r1 > r2 {
                let sha_targets = kongxin_sha_targets(ctx.core, ctx.catalog, owner, t);
                if sha_targets.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectNature;
                asks::wait_select_players(owner, "u05_kongxin_sha_target", sha_targets, true)
            } else {
                let pool = hand_or_equip(ctx.core, owner);
                if pool.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::CardsAsk;
                asks::wait_select_cards(t, "u05_kongxin_lose_discard", pool, 1, 1, false)
            }
        }
        HostStage::SelectNature => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            if let Some(ChoiceOption::Player(t2)) = ctx.answer().first().copied() {
                if ctx.core.player(t).alive
                    && ctx.core.player(t2).alive
                    && t2 != owner
                    && !crate::kernel::sha_user_or_target_blocked(ctx.core, ctx.catalog, t, t2)
                {
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: t,
                        targets: vec![t2],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
            }
            Flow::Done
        }
        HostStage::CardsAsk => {
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                discard_he_card(ctx, owner, c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_qinglonglingzhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let pool = hand_or_equip(ctx.core, t);
            if !ctx.core.player(t).alive || pool.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::SelectCard;
            asks::wait_select_cards(m.owner, "u05_qinglonglingzhu_gain", pool, 0, 1, true)
        }
        HostStage::SelectCard => {
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                prey_he_card(ctx, t, m.owner, c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_baihupifeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::SelectNature;
            asks::wait_confirm(m.owner, "u05_baihupifeng_recover")
        }
        HostStage::SelectNature => {
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                && ctx.core.player(m.owner).hp < ctx.core.player(m.owner).max_hp as i32
            {
                ctx.recover(m.owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_fengxueren(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            if !ctx.core.player(t).alive || ctx.core.player(t).face_down {
                return Flow::Done;
            }
            m.stage = HostStage::SelectNature;
            asks::wait_confirm(m.owner, "u05_fengxueren_confirm")
        }
        HostStage::SelectNature => {
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                && ctx.core.player(t).alive
                && !ctx.core.player(t).face_down
            {
                ctx.current_damage_mut().prevented = true;
                ctx.emit(EventData::TurnOverBefore {
                    player: t,
                    cancelled: false,
                });
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            if ctx.core.player(t).alive {
                ctx.start_draw(t, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_chilongya2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if ctx.core.rng.gen_range(2) == 0 {
        ctx.current_damage_mut().amount += 1;
        if let Some(t) = m.target {
            if ctx.core.player(t).alive {
                if let Some(def) = ctx.catalog.skill_by_name("chilongfengxue") {
                    ctx.ensure_skill(
                        (t, def),
                        crate::state::SkillInstance {
                            owner: t,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: true,
                            rounds: 0,
                        },
                    );
                }
            }
        }
    }
    Flow::Done
}

fn step_old_longfan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let opts: Vec<PlayerId> = ctx.core.alive_players().collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            m.stage = HostStage::TargetChoice;
            asks::wait_select_players(m.owner, "u05_old_longfan_target", opts, true)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                refund_host_once_used(ctx, m.owner, &m.def_key);
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            let roll = ctx.core.rng.gen_range(10);
            let same_team = ctx.core.is_friend_of(ctx.catalog, m.owner, t);
            match roll {
                0 => ctx.start_draw(t, 1),
                1 => random_discard_hand(ctx, t),
                2 => ctx.emit(EventData::LoseHp { player: t, amount: 1 }),
                3 => {
                    if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                        ctx.recover(t, 1, None);
                    }
                }
                4..=5 => {
                    if same_team {
                        ctx.start_draw(t, 1);
                    } else {
                        random_discard_hand(ctx, t);
                    }
                }
                6..=7 => {
                    if same_team {
                        if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                            ctx.recover(t, 1, None);
                        }
                    } else {
                        ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                    }
                }
                _ => random_discard_hand(ctx, m.owner),
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn discard_body_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) {
    if let Some(c) = m.card.take() {
        ctx.to_discard(c);
        ctx.core.table.retain(|x| *x != c);
    }
}

pub(super) fn black_hand_cards(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    core.player(p)
        .hand
        .iter()
        .copied()
        .filter(|c| {
            matches!(
                catalog.effective_card_suit(core, p, *c),
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            )
        })
        .collect()
}

pub(super) fn kongxin_sha_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
) -> Vec<PlayerId> {
    let range = core.attack_range(catalog, target);
    core.alive_players()
        .filter(|&t2| {
            t2 != owner
                && t2 != target
                && core.computed_distance(catalog, target, t2) <= range
                && !crate::kernel::sha_user_or_target_blocked(core, catalog, target, t2)
        })
        .collect()
}

pub(super) fn respondable_shans(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    crate::kernel::collect_shan_response_cards(core, catalog, p)
}

pub(super) fn prey_he_card(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId, card: CardId) {
    if ctx.core.player(from).hand.contains(&card) {
        ctx.give_hand_card(from, to, card);
    } else if let Some(pos) = ctx.core.player(from).equips.iter().position(|x| *x == card) {
        ctx.core.player_mut(from).equips.remove(pos);
        ctx.core.declared_slots.remove(&card);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, card);
        ctx.note_cards_lost(from, &[card]);
        ctx.emit(EventData::CardsLost {
            player: from,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Equip],
        });
        ctx.core.player_mut(to).hand.push(card);
        ctx.emit(EventData::CardsMoved {
            cards: vec![card],
            from,
            to,
        });
        ctx.log(LogEvent::CardsTaken {
            from,
            to,
            cards: vec![card],
            public: true,
        });
    }
}

fn random_discard_hand(ctx: &mut Ctx<'_>, p: PlayerId) {
    let len = ctx.core.player(p).hand.len();
    if len == 0 {
        return;
    }
    let idx = ctx.core.rng.gen_range(len as u32) as usize;
    let c = ctx.core.player(p).hand[idx];
    ctx.discard_from_hand(p, c);
}

pub(in crate::content::host) fn ask(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    asks::ask(behavior, stage, owner, target, cards, core, catalog)
}
