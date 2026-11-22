//! 内核固有机制：帧提交动作与内核流程机器。
//!
//! 原则：移动牌、扣血、摸牌、事件推进、濒死/死亡/出牌结算等**规则机制**
//! 归内核；具体技能的条件与流程归内容模块（`content`），两者通过同一套
//! 事件帧 + 机器接口协作。

use crate::catalog::{CardKind, SkillMachine};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{EventData, Phase};
use crate::ids::EventId;
use crate::ids::PlayerId;
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};

/// 帧提交阶段的一次性内核动作。
pub fn commit_frame(ctx: &mut Ctx<'_>, frame_id: EventId) {
    let (kind, data) = {
        let f = ctx.res.frame(frame_id).expect("commit frame");
        (f.kind, f.data.clone())
    };
    record_vm_history(ctx, &data);
    match &data {
        EventData::CardUsed(u) if !u.cancelled => record_turn_use_resp(ctx, u.player),
        EventData::CardResponded { player, .. } => record_turn_use_resp(ctx, *player),
        EventData::CardsLost { player, cards, .. } => clear_shilie_shown_on_leave(ctx, *player, cards),
        EventData::CardsMoved { from, cards, .. } => clear_shilie_shown_on_leave(ctx, *from, cards),
        _ => {}
    }
    match data {
        EventData::DamageWindow { .. } => {}
        EventData::LoseHp { player, amount } => {
            if ctx.catalog.vm_card_history && amount > 0 {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((player, "turn_change_hp".to_string()), 1);
            }
            commit_lose_hp(ctx, player, amount)
        }
        EventData::CardUsed(u) => commit_card_used(ctx, u),
        EventData::Equipped { player, card, .. } => commit_equip(ctx, frame_id, player, card),
        EventData::Linked { player, .. } => commit_linked(ctx, frame_id, player),
        EventData::Phase {
            player,
            phase,
            begin,
        } => commit_phase(ctx, player, phase, begin),
        EventData::TurnEnded { player } => commit_turn_ended(ctx, player),
        EventData::TurnBegin { player } => commit_turn_begin(ctx, player),
        EventData::TurnEndWindow { player, after } => {
            commit_turn_end_window(ctx, player, after)
        }
        EventData::TurnOverBefore { player, cancelled } => {
            commit_turn_over_before(ctx, player, cancelled)
        }
        EventData::CardsDiscarded { player, cards } => {
            commit_cards_discarded(ctx, player);
            if !cards.is_empty() {
                (ctx.catalog.hooks.on_cards_to_discard)(ctx, player, &cards, false);
            }
        }
        EventData::CardsMoved { cards, from, to } => {
            commit_cards_moved(ctx, to);
            for &c in &cards {
                if from != to || !ctx.core.player(from).hand.contains(&c) {
                    (ctx.catalog.hooks.on_hand_card_removed)(ctx, from, c);
                }
            }
        }
        EventData::Recover { player, .. } => commit_recover_skills(ctx, player),
        // CardsDrawn/Recover/CardsMoved/CardResponded 等在构造或 emit 时已完成日志/区移动作。
        _ => {
            let _ = kind;
        }
    }
}

fn clear_shilie_shown_on_leave(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    cards: &[crate::ids::CardId],
) {
    let has_shilie = ctx
        .catalog
        .skill_by_name("dddshilie")
        .is_some_and(|def| ctx.core.has_skill(player, def));
    for c in cards {
        if has_shilie {
            ctx.core
                .vm_aux
                .flags
                .remove(&(player, format!("dddshilie_shown_card:{c}")));
        }
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, format!("u01_shown_card:{c}")));
    }
}

pub(super) fn emit_card_responded_as(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: crate::ids::CardId,
    eff_kind: CardKind,
) {
    if ctx
        .catalog
        .skill_by_name("dddfushi")
        .is_some_and(|def| ctx.core.has_skill(player, def))
    {
        ctx.core.vm_aux.flags.insert(
            (player, "dddfushi_resp_eff_kind".to_string()),
            (eff_kind as u32) + 1,
        );
    }
    ctx.emit(EventData::CardResponded { card, player });
}

pub(super) fn record_virtual_respond(ctx: &mut Ctx<'_>, player: PlayerId, kind: CardKind) {
    record_turn_use_resp(ctx, player);
    record_dddanzhi_card(ctx.core, ctx.catalog, None);
    ctx.log(LogEvent::VirtualCardPlayed {
        kind,
        player,
        targets: Vec::new(),
        presentation: None,
    });
}

pub(super) fn record_turn_use_resp(ctx: &mut Ctx<'_>, actor: PlayerId) {
    let has_zhilian = ctx
        .catalog
        .skill_by_name("dddzhilian")
        .is_some_and(|def| ctx.core.alive_players().any(|p| ctx.core.has_skill(p, def)));
    if !has_zhilian {
        return;
    }
    let stamp = crate::rvm_bridge::modifiers::d34_l28_mods::l28_turn_stamp(ctx.core);
    let prev = if ctx
        .core
        .vm_aux
        .flags
        .get(&(0, "turn_use_resp_stamp".to_string()))
        .copied()
        .unwrap_or(0)
        == stamp
    {
        ctx.core
            .vm_aux
            .flags
            .get(&(0, "turn_use_resp_mask".to_string()))
            .copied()
            .unwrap_or(0)
    } else {
        0
    };
    ctx.core
        .vm_aux
        .flags
        .insert((0, "turn_use_resp_stamp".to_string()), stamp);
    ctx.core
        .vm_aux
        .flags
        .insert((0, "turn_use_resp_mask".to_string()), prev | (1u32 << actor));
}

mod damage;
// R05 抽牌流程机（D/R/G）。
mod draw;
mod damage_after;
mod damage_change_hp;
mod damage_flow;
mod damage_skill;
mod invocation_receipts;
mod jiding_card_changes;
pub use jiding_card_changes::{jiding_card_is_bound, record_jiding_card_change};
mod invocation_validation;
mod native_invocation;
pub use native_invocation::jiding_recovery_origin;
pub use native_invocation::{advance_invocation, NativeInvocation};
mod damage_windows;
use damage::*;
pub use damage_flow::DamageFlow;
pub use draw::{DrawMachine, DrawPhaseMachine, GainMachine};
pub use damage_skill::NativeDamageSkill;
mod events;
pub use events::{peek_tao_extra_carriers, record_dddanzhi_card, take_tao_extra_carriers};
use events::*;
mod equipment;
use equipment::commit_equip;
pub use equipment::{grant_equip_skills, revoke_equip_skills};
mod intrinsic;
pub use intrinsic::*;
mod combat_rules;
pub use combat_rules::{
    is_black_trick_card, jinyan_blocks_black_trick, mbcmfangzhu_baiban, mbcmfangzhu_banned,
    sha_card_use_as, tao_futai2_banned, usable_sha_options,
};
pub(crate) use combat_rules::*;
mod sha;
pub use sha::*;
mod dying;
pub use dying::*;
mod duel;
pub use duel::*;
mod tricks;
pub use tricks::*;
mod nullify;
pub use nullify::*;
mod judge;
pub use judge::*;
mod aoe;
pub use aoe::*;
mod discard;
pub use discard::*;
mod death;
pub use death::*;
mod monkey;
pub use monkey::*;
mod phase;
use phase::*;
pub mod elo;
pub use elo::*;
