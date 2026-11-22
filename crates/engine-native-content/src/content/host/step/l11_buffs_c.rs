//! L11（太虚幻境）Batch L11-4 Buffs Group C（`L11BuffsC` tag 736）：
//!
//! 承载 33 项 `extension_buff.js` / `extension_skill.js` 祝福与触发技：
//! - `luanjifengshi`（万箭齐发目标弃 1 牌或取消闪避）、`taoyuanyishi`（桃园己方回血 +1）
//! - `haoshidebaochao`/`baochang`（`l11_common_b4`）、`duocejiyong`（`nullify.rs`）
//! - `wolongmice`/`fenchengluetu`（`l11_cards*` + `tricks.rs`）、`_buff_txhj_jufuxionghao`（上游注释空操作）
//! - `bowenqiangzhi`/`majundegaizao`/`xianhujiqu`（`taixu_campaign_data.rs`）
//! - `fuchendangmo`/`shangjiangdelvli`/`shenhanxueyin`（武器修饰见 `d34_l11_buff_mods.rs`）
//! - `shenfuhuaxie`（受伤后拾取桌面造成伤害牌，无奸雄/re奸雄）、`gongzhibaowu`（准备阶段有马得杀）
//! - `fangzhibaowu`（结束阶段有马得闪）、`suikuiliejia`（持武器杀伤后可选弃目标防具）
//! - `yiguzuoqi`（出牌阶段第 2 张杀摸 1）、`xiuluodebeimin`/`lianmin`（我引发回复时持武器摸 1）
//! - `cebeibuyu`（有防具受锦囊伤 -1）、`gongtaizhizhi`（每轮开始满血有防具摸 1）

use super::l11_common::*;
use super::l11_common_b2::damage_effective_kind;
use super::l11_common_b3::{gain_first_from_deck_or_discard, has_armor_slot, has_weapon_slot};
use super::l11_common_b4::*;
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(in crate::content::host) fn trigger_meta_l11_buffs_c(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, optional: bool, extra_events| {
        TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional,
            extra_events,
        }
    };
    match name {
        "buff_txhj_luanjifengshi"
        | "_buff_txhj_luanjifengshi"
        | "buff_txhj_taoyuanyishi"
        | "_buff_txhj_taoyuanyishi"
        | "buff_txhj_yiguzuoqi"
        | "_buff_txhj_yiguzuoqi" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_shenfuhuaxie"
        | "_buff_txhj_shenfuhuaxie"
        | "buff_txhj_suikuiliejia"
        | "_buff_txhj_suikuiliejia" => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_gongzhibaowu"
        | "_buff_txhj_gongzhibaowu"
        | "buff_txhj_fangzhibaowu"
        | "_buff_txhj_fangzhibaowu" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_xiuluodebeimin" | "_buff_txhj_xiuluodelianmin" => Some(meta(
            EventKind::Recover,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_cebeibuyu" | "_buff_txhj_cebeibuyu" => Some(meta(
            EventKind::DamageBegin3,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_gongtaizhizhi" | "_buff_txhj_gongtaizhizhi" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_buffs_c(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_buffs_c(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

fn has_horse_slot(state: &GameStateCore, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
    state.player(p).equips.iter().any(|&c| {
        matches!(
            catalog.card_slot_of(state, c),
            Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
        )
    })
}

fn has_jianxiong(state: &GameStateCore, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
    has_named_skill(state, catalog, p, "jianxiong")
        || has_named_skill(state, catalog, p, crate::vocabulary::names::JIAN_XIONG)
        || has_named_skill(state, catalog, p, "rejianxiong")
        || has_named_skill(state, catalog, p, "re_jianxiong")
}

pub(in crate::content::host) fn filter_l11_buffs_c_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
        || buff_yields_to_companion_b4(state, ctx.catalog, owner, &skill.name)
    {
        return false;
    }
    match skill.name.as_str() {
        "buff_txhj_luanjifengshi" | "_buff_txhj_luanjifengshi" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog) == CardKind::ArcheryAttack
                    && !is_card_named(ctx.catalog, u.card, "txhj_huoshaolianying")
                    && !u.targets.is_empty()
        ),
        "buff_txhj_taoyuanyishi" | "_buff_txhj_taoyuanyishi" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog) == CardKind::Taoyuan
        ),
        "buff_txhj_shenfuhuaxie" | "_buff_txhj_shenfuhuaxie" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && !has_jianxiong(state, ctx.catalog, owner)
                    && d.card.is_some_and(|cid| state.table.contains(&cid))
        ),
        "buff_txhj_gongzhibaowu" | "_buff_txhj_gongzhibaowu" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
                if *player == owner && has_horse_slot(state, ctx.catalog, owner)
        ),
        "buff_txhj_fangzhibaowu" | "_buff_txhj_fangzhibaowu" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner && has_horse_slot(state, ctx.catalog, owner)
        ),
        "buff_txhj_suikuiliejia" | "_buff_txhj_suikuiliejia" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.from_chain
                    && !d.prevented
                    && d.amount > 0
                    && state.player(d.target).alive
                    && has_weapon_slot(state, ctx.catalog, owner)
                    && has_armor_slot(state, ctx.catalog, d.target)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(CardKind::is_sha_like)
        ),
        "buff_txhj_yiguzuoqi" | "_buff_txhj_yiguzuoqi" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && state.turn.current == owner
                    && state.turn.phase == Phase::Play
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && has_weapon_slot(state, ctx.catalog, owner)
                    && flag(state, owner, "l11_phase_sha_count") == 2
        ),
        "buff_txhj_xiuluodebeimin" | "_buff_txhj_xiuluodelianmin" => match event {
            EventData::Recover { player, .. } => {
                has_weapon_slot(state, ctx.catalog, owner)
                    && frame.is_some_and(|cf| {
                        flag(state, *player, &format!("l11_recover_source:{}", cf.id))
                            == u32::from(owner) + 1
                    })
            }
            _ => false,
        },
        "buff_txhj_cebeibuyu" | "_buff_txhj_cebeibuyu" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && has_armor_slot(state, ctx.catalog, owner)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(crate::rvm_bridge::modifiers::is_trick_like_kind)
        ),
        "buff_txhj_gongtaizhizhi" | "_buff_txhj_gongtaizhizhi" => matches!(
            event,
            EventData::TurnBegin { .. }
                if flag(state, owner, "l11_gongtaizhizhi_round") != state.turn.round + 1
                    && has_armor_slot(state, ctx.catalog, owner)
                    && state.player(owner).hp >= state.player(owner).max_hp as i32
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_buffs_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, card, targets) = match event {
        EventData::Damage(d) => (Some(d.target), d.card, Vec::new()),
        EventData::CardUsed(u) => (None, u.card, u.targets.clone()),
        _ => (None, None, Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11BuffsC,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_buffs_c(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "buff_txhj_luanjifengshi" | "_buff_txhj_luanjifengshi" => handle_luanjifengshi(m, ctx),
        "buff_txhj_taoyuanyishi" | "_buff_txhj_taoyuanyishi" => {
            if !has_named_skill(ctx.core, ctx.catalog, owner, "txhj_taoyuanyishi") {
                ensure_perm_skill(ctx, owner, "txhj_taoyuanyishi");
                set_flag(ctx.core, owner, "l11_temp_taoyuanyishi", 1);
            }
            let order = alive_others(ctx.core, owner);
            for p in order {
                if ctx.core.is_friend_of(ctx.catalog, owner, p)
                    && !has_named_skill(ctx.core, ctx.catalog, p, "taoyuanyishiX")
                {
                    ensure_perm_skill(ctx, p, "taoyuanyishiX");
                    set_flag(ctx.core, p, "l11_temp_taoyuanyishi_x", 1);
                }
            }
            Flow::Done
        }
        "buff_txhj_shenfuhuaxie" | "_buff_txhj_shenfuhuaxie" => {
            if let Some(cid) = m.card {
                if ctx.core.table.contains(&cid) && ctx.core.player(owner).alive {
                    ctx.core.table.retain(|&x| x != cid);
                    let delivered = (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &[cid]);
                    ctx.core.player_mut(owner).hand.extend(delivered.iter().copied());
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: owner,
                            cards: delivered.clone(),
                        });
                        ctx.emit(EventData::CardsDrawn {
                            player: owner,
                            cards: delivered,
                        });
                    }
                }
            }
            Flow::Done
        }
        "buff_txhj_gongzhibaowu" | "_buff_txhj_gongzhibaowu" => {
            let _ = gain_first_from_deck_or_discard(ctx, owner, CardKind::is_sha_like);
            Flow::Done
        }
        "buff_txhj_fangzhibaowu" | "_buff_txhj_fangzhibaowu" => {
            let _ = gain_first_from_deck_or_discard(ctx, owner, |k| k == CardKind::Shan);
            Flow::Done
        }
        "buff_txhj_suikuiliejia" | "_buff_txhj_suikuiliejia" => handle_suikuiliejia(m, ctx),
        "buff_txhj_yiguzuoqi" | "_buff_txhj_yiguzuoqi" => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "buff_txhj_xiuluodebeimin" | "_buff_txhj_xiuluodelianmin" => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "buff_txhj_cebeibuyu" | "_buff_txhj_cebeibuyu" => {
            with_current_damage_mut(ctx, |dm| {
                dm.amount = dm.amount.saturating_sub(1);
                if dm.amount == 0 {
                    dm.prevented = true;
                }
            });
            Flow::Done
        }
        "buff_txhj_gongtaizhizhi" | "_buff_txhj_gongtaizhizhi" => {
            let round_stamp = ctx.core.turn.round + 1;
            set_flag(ctx.core, owner, "l11_gongtaizhizhi_round", round_stamp);
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_luanjifengshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cid = m.card.unwrap_or(0);
    if m.stage == HostStage::SelectCard {
        let mut yielded_for_discard = false;
        if let Some(t) = m.target {
            let picked = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if he_cards(ctx.core, t).contains(&c) => Some(c),
                _ => None,
            };
            if let Some(c) = picked {
                discard_player_cards(ctx, t, &[c]);
                yielded_for_discard = true;
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((t, format!("aoe_direct_hit:{cid}")), 1);
            }
        }
        if !m.targets.is_empty() {
            m.targets.remove(0);
        }
        m.target = None;
        m.stage = HostStage::Act;
        if yielded_for_discard {
            return Flow::Yield;
        }
    }
    while let Some(&t) = m.targets.first() {
        if !ctx.core.player(t).alive
            || crate::kernel::dushiwang_remen_blocks_core(ctx.core, ctx.catalog, t)
            || crate::kernel::weimu_blocks_aoe_core(ctx.core, ctx.catalog, cid, t)
            || ctx.catalog.aoe_excludes_target(ctx.core, t)
            || crate::rvm_bridge::modifiers::shierqicejin_excludes(
                ctx.core,
                ctx.catalog,
                m.owner,
                t,
            )
            || crate::rvm_bridge::modifiers::eval_card_target_canceled(
                ctx.core,
                ctx.catalog,
                m.owner,
                t,
                cid,
                CardKind::ArcheryAttack,
                DamageNature::Normal,
            )
        {
            m.targets.remove(0);
            continue;
        }
        if he_cards(ctx.core, t).is_empty() {
            ctx.core
                .vm_aux
                .flags
                .insert((t, format!("aoe_direct_hit:{cid}")), 1);
            m.targets.remove(0);
            continue;
        }
        m.target = Some(t);
        m.stage = HostStage::SelectCard;
        if let Some((observer, spec)) = super::l11_buffs_b4_asks::build_l11_buffs_c_ask(
            m.stage,
            m.owner,
            m.target,
            ctx.core,
            ctx.catalog,
        ) {
            return Flow::WaitChoice { observer, spec };
        }
        m.target = None;
        m.targets.remove(0);
    }
    Flow::Done
}

fn handle_suikuiliejia(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
        return Flow::Done;
    };
    if m.stage == HostStage::TargetChoice {
        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
            if ctx.core.player(t).equips.contains(&c)
                && ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Armor)
            {
                discard_player_cards(ctx, t, &[c]);
            }
        }
        return Flow::Done;
    }
    m.stage = HostStage::TargetChoice;
    if let Some((observer, spec)) = super::l11_buffs_b4_asks::build_l11_buffs_c_ask(
        m.stage,
        m.owner,
        m.target,
        ctx.core,
        ctx.catalog,
    ) {
        return Flow::WaitChoice { observer, spec };
    }
    Flow::Done
}
