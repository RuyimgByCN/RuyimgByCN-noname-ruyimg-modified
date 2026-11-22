//! L11（太虚幻境）Batch L11-4 Buffs Group D（`L11BuffsD` tag 737）：
//!
//! 承载 37 项 `extension_buff.js` / `extension_skill.js` 祝福与触发技：
//! - `tianxiawushuang`（普通【杀】伤害 +1）、`jijianchangge`（回合外使用或打出【杀】摸 1）
//! - `huangtianzhinu`（属性伤害后随机弃置目标 `he` 1 张牌）、`luoshenjuezhang`（授予 `txhj_luoshenjuezhang`）
//! - `shenyizhidao`（使用【桃】回复量 +1）、`mowangdejianiang`（`l11_common_b4` 授衍生+叠层）
//! - `hannu`（`l11_common_b4` 用酒后取杀）、`gusoejueyan`/`guosejueyan`（`on_judge_settled_l11_b4`）
//! - `mingyundezhiyi`（`l11_common_b4` 闪电后横置未横置敌人）、`ximengzhili`（`l11_cards.rs` 远交+1）
//! - `hengxingwuji`（`l11_common_b4` 过拆后摸 1）
//! - `shierqicejin`/`shierqicetui`（授辅助标记 + `aoe.rs`/`tricks.rs` 排除）
//! - `pofuchenzhou`（初始手牌=1，若同时有 `fuzu` 则被 `fuzu` 覆写为体力上限，并授 `txhj_pofuchenzhou`）
//! - `huangtianyili`/`huangtianzhizu`（`apply_huangtian_suit_rank`）、`huangtianzhimin`（`on_judge_settled_l11_b4`）
//! - `tiangongdeyouhu`/`digongdeyouhu`/`renigongdeyouhu`（`l27_batch_b*` & `decision34_l11_skills.rs`）

use super::l11_common::*;
use super::l11_common_b2::damage_effective_kind;
use super::l11_common_b3::random_discard_he;
use super::l11_common_b4::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(in crate::content::host) fn trigger_meta_l11_buffs_d(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "buff_txhj_tianxiawushuang" | "_buff_txhj_tianxiawushuang" => Some(meta(
            EventKind::DamageBegin1,
            Timing::Normal,
            0,
            Vec::new(),
        )),
        "buff_txhj_jijianchangge" | "_buff_txhj_jijianchangge" => Some(meta(
            EventKind::CardResponded,
            Timing::Normal,
            0,
            vec![EventKind::CardUsed],
        )),
        "buff_txhj_huangtianzhinu" | "_buff_txhj_huangtianzhinu" => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_luoshenjuezhang"
        | "_buff_txhj_luoshenjuezhang"
        | "buff_txhj_shierqicejin"
        | "_buff_txhj_shierqicejin"
        | "buff_txhj_shierqicetui"
        | "_buff_txhj_shierqicetui" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            0,
            Vec::new(),
        )),
        "buff_txhj_pofuchenzhou" | "_buff_txhj_pofuchenzhou" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            7,
            Vec::new(),
        )),
        "buff_txhj_shenyizhidao" | "_buff_txhj_shenyizhidao" => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 0, Vec::new()))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_buffs_d(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_buffs_d(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

pub(in crate::content::host) fn filter_l11_buffs_d_with_frames(
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
        "buff_txhj_tianxiawushuang" | "_buff_txhj_tianxiawushuang" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.from_chain
                    && !d.prevented
                    && d.nature == DamageNature::Normal
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(CardKind::is_sha_like)
        ),
        "buff_txhj_jijianchangge" | "_buff_txhj_jijianchangge" => {
            if state.turn.current == owner {
                return false;
            }
            match event {
                EventData::CardUsed(u) => {
                    u.player == owner && !u.cancelled && u.effective_kind(ctx.catalog).is_sha_like()
                }
                EventData::CardResponded { card, player } => {
                    *player == owner && effective_card_kind(state, ctx.catalog, *card).is_sha_like()
                }
                _ => false,
            }
        }
        "buff_txhj_huangtianzhinu" | "_buff_txhj_huangtianzhinu" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != DamageNature::Normal
                    && state.player(d.target).alive
                    && !he_cards(state, d.target).is_empty()
        ),
        "buff_txhj_luoshenjuezhang" | "_buff_txhj_luoshenjuezhang" => {
            matches!(event, EventData::TurnBegin { .. })
                && flag(state, owner, "l11_luoshenjuezhang_granted") == 0
                && !has_named_skill(state, ctx.catalog, owner, "txhj_luoshenjuezhang")
        }
        "buff_txhj_shenyizhidao" | "_buff_txhj_shenyizhidao" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog) == CardKind::Tao
        ),
        "buff_txhj_shierqicejin" | "_buff_txhj_shierqicejin" => {
            matches!(event, EventData::TurnBegin { .. })
                && flag(state, owner, "l11_shierqicejin_applied") == 0
        }
        "buff_txhj_shierqicetui" | "_buff_txhj_shierqicetui" => {
            matches!(event, EventData::TurnBegin { .. })
                && flag(state, owner, "l11_shierqicetui_applied") == 0
        }
        "buff_txhj_pofuchenzhou" | "_buff_txhj_pofuchenzhou" => {
            matches!(event, EventData::TurnBegin { .. })
                && flag(state, owner, "l11_pofuchenzhou_init") == 0
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_buffs_d(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let target = match event {
        EventData::Damage(d) => Some(d.target),
        _ => None,
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11BuffsD,
        owner,
        target,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_buffs_d(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "buff_txhj_tianxiawushuang" | "_buff_txhj_tianxiawushuang" => {
            with_current_damage_mut(ctx, |dm| {
                dm.amount += 1;
            });
            Flow::Done
        }
        "buff_txhj_jijianchangge" | "_buff_txhj_jijianchangge" => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "buff_txhj_huangtianzhinu" | "_buff_txhj_huangtianzhinu" => {
            if let Some(t) = m.target {
                random_discard_he(ctx, t);
            }
            Flow::Done
        }
        "buff_txhj_luoshenjuezhang" | "_buff_txhj_luoshenjuezhang" => {
            set_flag(ctx.core, owner, "l11_luoshenjuezhang_granted", 1);
            ensure_perm_skill(ctx, owner, "txhj_luoshenjuezhang");
            Flow::Done
        }
        "buff_txhj_shenyizhidao" | "_buff_txhj_shenyizhidao" => {
            let prev = flag(ctx.core, owner, "vm_va_recover_bonus");
            set_flag(ctx.core, owner, "vm_va_recover_bonus", prev + 1);
            Flow::Done
        }
        "buff_txhj_shierqicejin" | "_buff_txhj_shierqicejin" => {
            set_flag(ctx.core, owner, "l11_shierqicejin_applied", 1);
            let order = alive_others(ctx.core, owner);
            for p in order {
                if ctx.core.is_friend_of(ctx.catalog, owner, p) {
                    ensure_perm_skill(ctx, p, "txhj_shierqicejin");
                }
            }
            Flow::Done
        }
        "buff_txhj_shierqicetui" | "_buff_txhj_shierqicetui" => {
            set_flag(ctx.core, owner, "l11_shierqicetui_applied", 1);
            let order = alive_others(ctx.core, owner);
            for p in order {
                if !ctx.core.is_friend_of(ctx.catalog, owner, p) {
                    ensure_perm_skill(ctx, p, "txhj_shierqicetui");
                }
            }
            Flow::Done
        }
        "buff_txhj_pofuchenzhou" | "_buff_txhj_pofuchenzhou" => {
            set_flag(ctx.core, owner, "l11_pofuchenzhou_init", 1);
            ensure_perm_skill(ctx, owner, "txhj_pofuchenzhou");
            let has_fuzu = has_named_skill(ctx.core, ctx.catalog, owner, "buff_txhj_fuzu")
                || has_named_skill(ctx.core, ctx.catalog, owner, "_buff_txhj_fuzu");
            if !has_fuzu {
                let hand_len = ctx.core.player(owner).hand.len();
                if hand_len == 0 {
                    ctx.start_draw(owner, 1);
                } else if hand_len > 1 {
                    while ctx.core.player(owner).hand.len() > 1 {
                        if let Some(c) = ctx.core.player_mut(owner).hand.pop() {
                            ctx.core.deck.push(c);
                        }
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
