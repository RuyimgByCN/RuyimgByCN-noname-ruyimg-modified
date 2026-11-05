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
use crate::machine::{damage_data, damage_data_with, Ctx, Flow, LogEvent};

/// 帧提交阶段的一次性内核动作。
pub fn commit_frame(ctx: &mut Ctx<'_>, frame_id: EventId) {
    let (kind, data) = {
        let f = ctx.res.frame(frame_id).expect("commit frame");
        (f.kind, f.data.clone())
    };
    record_vm_history(ctx, &data);
    match data {
        EventData::DamageWindow { .. } => {}
        EventData::LoseHp { player, amount } => commit_lose_hp(ctx, player, amount),
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
        EventData::CardsDiscarded { player, .. } => commit_cards_discarded(ctx, player),
        EventData::CardsMoved { to, .. } => commit_cards_moved(ctx, to),
        EventData::Recover { player, .. } => commit_recover_skills(ctx, player),
        // CardsDrawn/Recover/CardsMoved/CardResponded 等在构造或 emit 时已完成日志/区移动作。
        _ => {
            let _ = kind;
        }
    }
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
use events::*;
mod equipment;
use equipment::commit_equip;
pub use equipment::{grant_equip_skills, revoke_equip_skills};
mod intrinsic;
pub use intrinsic::*;
mod combat_rules;
pub use combat_rules::tao_futai2_banned;
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
