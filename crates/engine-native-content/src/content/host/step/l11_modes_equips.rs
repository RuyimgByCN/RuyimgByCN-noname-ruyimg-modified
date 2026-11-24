//! L11 太虚幻境 DLC 与 `extension_card.js` 装备技能宿主实现（`HostBehavior::L11ModesEquips`，tag 1395）。
//!
//! 覆盖 5 件真实上游装备技能：
//! - `txhj_zhenhunqin`（`extension_card.js:1326,2287-2304`）：使用普通【杀】时可令其属性变为雷电；
//! - `txhj_xieshenmianju`（`extension_card.js:1339,2306-2337`）：锁定技，不能翻面；受到 >= 2 点伤害时 -1；
//! - `txhj_shengguangbaiyi`（`extension_card.js:1275,2183-2214`）：锁定技，红色【杀】无效，手牌上限 +2；
//! - `nvzhuang`（`yongjian.js:220-253,360-390`）：锁定技，男性成为【杀】目标后判定，若为黑色则此【杀】伤害基数 +1；
//! - `yajiaoqiang_skill`（`zhulu.js:360-381,824-838`）：其他角色回合内首次使用的黑色牌结算后可获得其实体牌。

use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog, Suit, TriggerMeta};
use crate::choice::ChoiceSpec;
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Frame, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::rvm_bridge::modifiers::{d34_l28_mods, is_target_unequip2};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

const ASK_YAJIAOQIANG_CONFIRM: u32 = 1;
const ASK_ZHENHUNQIN_CONFIRM: u32 = 2;

fn wait_meq_ask(
    m: &mut HostMachine,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_modes_equips_ask(ask_id, m.owner, m.target) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        Flow::Done
    }
}

fn build_l11_modes_equips_ask(
    ask_id: u32,
    owner: PlayerId,
    _target: Option<PlayerId>,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_YAJIAOQIANG_CONFIRM => confirm_ask(owner, "yajiaoqiang_skill:confirm"),
        ASK_ZHENHUNQIN_CONFIRM => confirm_ask(owner, "txhj_zhenhunqin:confirm"),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l11_modes_equips(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<CardId>,
    _cards: &[CardId],
    _core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle {
        return None;
    }
    let ask_id = card?;
    let (observer, spec) = build_l11_modes_equips_ask(ask_id, owner, target)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

pub(in crate::content::host) fn trigger_meta_l11_modes_equips(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events: Vec<EventKind>| {
        TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional: false,
            extra_events,
        }
    };
    match name {
        "txhj_zhenhunqin" => Some(meta(EventKind::CardUsed, Timing::Normal, 7, Vec::new())),
        "txhj_shengguangbaiyi" => Some(meta(EventKind::CardUsed, Timing::Normal, 8, Vec::new())),
        "nvzhuang" => Some(meta(EventKind::CardUsed, Timing::Normal, 5, Vec::new())),
        "txhj_xieshenmianju" => Some(meta(
            EventKind::TurnOverBefore,
            Timing::Normal,
            1,
            vec![EventKind::DamageBegin4],
        )),
        "yajiaoqiang_skill" => Some(meta(EventKind::CardResponded, Timing::Normal, -10, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_modes_equips(_name: &str) -> crate::catalog::SkillLimits {
    crate::catalog::SkillLimits::default()
}

fn is_red_use_card(state: &GameStateCore, catalog: &Catalog, u: &CardUseData) -> bool {
    let Some(cid) = u.card else {
        return false;
    };
    if flag(state, u.player, &format!("swd_linyun_colorless:{cid}")) > 0 {
        return false;
    }
    matches!(
        catalog.effective_card_suit(state, u.player, cid),
        Suit::Heart | Suit::Diamond
    )
}

fn is_black_card_id(state: &GameStateCore, catalog: &Catalog, user: PlayerId, cid: CardId) -> bool {
    if flag(state, user, &format!("swd_linyun_colorless:{cid}")) > 0 {
        return false;
    }
    matches!(
        catalog.effective_card_suit(state, user, cid),
        Suit::Spade | Suit::Club
    )
}

fn is_black_use_card(state: &GameStateCore, catalog: &Catalog, u: &CardUseData) -> bool {
    u.card
        .is_some_and(|cid| is_black_card_id(state, catalog, u.player, cid))
}

fn seat_is_male_l11(state: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    let slots = &state.player(p).general_slots;
    if slots.iter().all(|s| s.is_none()) {
        return false;
    }
    state.matches_gender(catalog, p, crate::content::standard::Gender::Male)
        || (flag(state, p, "effective_gender_override") == 0
            && slots.iter().flatten().any(|&g| {
                crate::content::standard::general_gender(g) == Some(crate::content::standard::Gender::Male)
                    || catalog.general_meta(g).is_some_and(|(_, _, _, _, gen)| {
                        gen == crate::content::standard::Gender::Male
                    })
            }))
}

fn armor_active_against_use(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    u: &CardUseData,
) -> bool {
    !is_target_unequip2(state, owner)
        && !d34_l28_mods::source_ignores_armor(
            state,
            catalog,
            u.player,
            owner,
            u.card,
            effective_kind_of_use(catalog, u),
            u.nature,
        )
}

pub(in crate::content::host) fn filter_l11_modes_equips_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    _frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "txhj_zhenhunqin" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player == owner
                && !u.cancelled
                && !u.targets.is_empty()
                && u.nature == DamageNature::Normal
                && effective_kind_of_use(catalog, u) == Some(CardKind::Sha)
        }
        "txhj_shengguangbaiyi" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.targets.contains(&owner)
                && !u.cancelled
                && is_sha_use(catalog, u)
                && is_red_use_card(state, catalog, u)
                && armor_active_against_use(state, catalog, owner, u)
        }
        "nvzhuang" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.targets.contains(&owner)
                && !u.cancelled
                && is_sha_use(catalog, u)
                && seat_is_male_l11(state, catalog, owner)
                && armor_active_against_use(state, catalog, owner, u)
        }
        "txhj_xieshenmianju" => match event {
            EventData::TurnOverBefore { player, cancelled } => {
                *player == owner && !*cancelled && !is_target_unequip2(state, owner)
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount >= 2
                    && !is_target_unequip2(state, owner)
                    && !d.source.is_some_and(|s| {
                        d34_l28_mods::source_ignores_armor(
                            state, catalog, s, owner, d.card, None, d.nature,
                        )
                    })
            }
            _ => false,
        },
        "yajiaoqiang_skill" => {
            let EventData::CardResponded { card, player } = event else {
                return false;
            };
            if *player != owner
                || state.turn.current == owner
                || catalog.is_tieji_sealed(state, owner)
                || flag(state, owner, "skill_turn:l11_yajiaoqiang_black_used") != 0
                || (!state.table.contains(card) && !state.discard.contains(card))
                || !is_black_card_id(state, catalog, owner, *card)
            {
                return false;
            }
            let eff_kind = catalog.resolved_card_kind(state, *card);
            if eff_kind == CardKind::Nullify {
                return true;
            }
            let nearest_use = frames.iter().rev().find(|f| f.card_use().is_some());
            let in_aoe_shan_respond = nearest_use
                .and_then(|f| f.card_use())
                .and_then(|u| effective_kind_of_use(catalog, u))
                .is_some_and(|k| k.aoe_need() == Some(CardKind::Shan));
            if in_aoe_shan_respond {
                return false;
            }
            let is_sha_dodge = nearest_use.is_some_and(|f| {
                flag(state, owner, &format!("l11_sha_dodge_ok:{}:{}", f.id, *card)) > 0
            });
            eff_kind == CardKind::Shan || is_sha_dodge
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_modes_equips(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, card, cards, targets) = match event {
        EventData::CardUsed(u) => (Some(u.player), u.card, Vec::new(), u.targets.clone()),
        EventData::CardResponded { card, player } => {
            (Some(*player), Some(*card), vec![*card], Vec::new())
        }
        EventData::Damage(d) => (Some(d.target), d.card, Vec::new(), Vec::new()),
        _ => (None, None, Vec::new(), Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11ModesEquips,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn activate_l11_modes_equips(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    None
}

pub(in crate::content::host) fn handle_l11_modes_equips(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "txhj_zhenhunqin" => handle_txhj_zhenhunqin(m, ctx),
        "txhj_shengguangbaiyi" => handle_txhj_shengguangbaiyi(ctx, owner),
        "txhj_xieshenmianju" => handle_txhj_xieshenmianju(ctx),
        "nvzhuang" => handle_nvzhuang(m, ctx),
        "yajiaoqiang_skill" => handle_yajiaoqiang(m, ctx),
        _ => Flow::Done,
    }
}

fn handle_txhj_zhenhunqin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_meq_ask(m, HostStage::TargetChoice, ASK_ZHENHUNQIN_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if is_confirmed(ctx) {
                if let Some(u) = current_use_card_mut(ctx) {
                    if u.nature == DamageNature::Normal {
                        u.nature = DamageNature::Thunder;
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_txhj_shengguangbaiyi(ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    if let Some(u) = current_use_card_mut(ctx) {
        u.targets.retain(|&p| p != owner);
        if u.targets.is_empty() {
            u.cancelled = true;
        }
    }
    Flow::Done
}

fn handle_txhj_xieshenmianju(ctx: &mut Ctx<'_>) -> Flow {
    let fid = ctx.current_event_frame_id();
    let is_turnover = ctx
        .res
        .frame(fid)
        .is_some_and(|f| matches!(f.data, EventData::TurnOverBefore { .. }));
    if is_turnover {
        let (_, cancelled) = ctx.current_turn_over_before_mut();
        *cancelled = true;
        return Flow::Done;
    }
    with_current_damage_mut(ctx, |dm| {
        if dm.amount >= 2 {
            dm.amount -= 1;
        }
    });
    Flow::Done
}

fn handle_nvzhuang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let cid = m.card.unwrap_or(0);
            m.cards = vec![cid];
            m.card = None;
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let suit_code = flag(ctx.core, owner, "last_judge_suit");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            set_flag(ctx.core, owner, "last_judge_card", 0);
            if matches!(suit_code, 1 | 3) {
                if let Some(attacker) = m.target {
                    let cid = m.cards.first().copied().unwrap_or(0);
                    let bonus_key = format!("sha_target_damage_bonus:{cid}:{owner}");
                    let prev = flag(ctx.core, attacker, &bonus_key);
                    set_flag(ctx.core, attacker, &bonus_key, prev + 1);
                    set_flag(ctx.core, attacker, &format!("l11_nvzhuang_set:{cid}:{owner}"), 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_yajiaoqiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.target.take().is_some() {
                let prev = flag(ctx.core, owner, "skill_turn:l11_yajiaoqiang_black_used");
                if prev != 0 {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "skill_turn:l11_yajiaoqiang_black_used", 1);
            }
            let Some(cid) = m.cards.first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.table.contains(&cid) && !ctx.core.discard.contains(&cid) {
                return Flow::Done;
            }
            wait_meq_ask(m, HostStage::TargetChoice, ASK_YAJIAOQIANG_CONFIRM)
        }
        HostStage::TargetChoice => {
            m.card = None;
            if is_confirmed(ctx) {
                if let Some(cid) = m.cards.first().copied() {
                    let mut taken = false;
                    if let Some(pos) = ctx.core.table.iter().position(|&c| c == cid) {
                        ctx.core.table.remove(pos);
                        taken = true;
                    } else if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
                        ctx.core.discard.remove(pos);
                        taken = true;
                    }
                    if taken && !ctx.core.player(owner).hand.contains(&cid) {
                        ctx.core.player_mut(owner).hand.push(cid);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `useCardAfter` 生命周期钩子：
/// 1. 清理 `nvzhuang` 产生的单次伤害基数旗标，避免跨虚拟杀残留；
/// 2. `yajiaoqiang_skill`（`zhulu.js:824-838`）：于其他角色回合内首次使用黑色牌结算后，若实体牌仍在处理区/弃牌堆则可获得。
pub(super) fn on_card_use_after_l11_modes(ctx: &mut Ctx<'_>, u: &CardUseData) {
    if ctx.catalog.skill_by_name("nvzhuang").is_some() && is_sha_use(ctx.catalog, u) {
        let cid = u.card.unwrap_or(0);
        for &t in &u.targets {
            let set_key = format!("l11_nvzhuang_set:{cid}:{t}");
            if flag(ctx.core, u.player, &set_key) > 0 {
                set_flag(ctx.core, u.player, &set_key, 0);
                set_flag(
                    ctx.core,
                    u.player,
                    &format!("sha_target_damage_bonus:{cid}:{t}"),
                    0,
                );
            }
        }
    }

    if ctx.catalog.skill_by_name("yajiaoqiang_skill").is_none() {
        return;
    }
    if !is_black_use_card(ctx.core, ctx.catalog, u) {
        return;
    }
    let owner = u.player;
    let prev = flag(ctx.core, owner, "skill_turn:l11_yajiaoqiang_black_used");
    set_flag(
        ctx.core,
        owner,
        "skill_turn:l11_yajiaoqiang_black_used",
        prev + 1,
    );
    if prev != 0
        || ctx.core.turn.current == owner
        || !ctx.core.player(owner).alive
        || !has_named_skill(ctx.core, ctx.catalog, owner, "yajiaoqiang_skill")
        || ctx.catalog.is_tieji_sealed(ctx.core, owner)
    {
        return;
    }
    let Some(cid) = u.card else {
        return;
    };
    if !ctx.core.table.contains(&cid) && !ctx.core.discard.contains(&cid) {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "yajiaoqiang_skill".to_string(),
            behavior: HostBehavior::L11ModesEquips,
            owner,
            target: None,
            card: None,
            cards: vec![cid],
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
}
