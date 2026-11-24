//! L11-6a 太虚自有 NPC 技能事件过滤（`l11_own_npc/filter.rs`）。

use super::*;
use crate::catalog::{QueryCtx, Suit};
use crate::event::{DamageNature, EventData, Frame, Phase, Stage};
use super::super::l11_common_b2::damage_effective_kind;

pub(in crate::content::host) fn filter_l11_own_npc(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    match skill.name.as_str() {
        "txhj_zuijiu" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !d.from_chain
                    && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(crate::catalog::CardKind::is_sha_like)
        ),
        "txhj_baoji" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !d.from_chain
                    && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(crate::catalog::CardKind::is_sha_like)
        ),
        "txhj_kuangbao" => {
            if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner) {
                return false;
            }
            match event {
                EventData::Dying { player, .. } => *player == owner,
                EventData::Damage(d) => {
                    d.source == Some(owner)
                        && !d.prevented
                        && d.amount > 0
                        && flag(state, owner, "l11_kuangbao_bonus") > 0
                }
                _ => false,
            }
        }
        "txhj_shiyuan" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
        ),
        "txhj_wangyuan" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !d.from_chain
                    && d.nature != DamageNature::Normal
                    && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                    && flag(state, owner, "l11_wangyuan_marks") > 0
        ),
        "txhj_guiyan" => {
            if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner) {
                return false;
            }
            match event {
                EventData::Phase { player, phase: Phase::Prepare, begin: true } => {
                    *player == owner
                        && alive_others(state, owner)
                            .into_iter()
                            .any(|p| state.player(p).hp >= state.player(owner).hp)
                }
                EventData::Damage(d) => {
                    d.source == Some(owner)
                        && d.target != owner
                        && !d.prevented
                        && d.amount > 0
                        && flag(state, owner, "l11_guiyan_target") == u32::from(d.target) + 1
                        && flag(state, owner, "l11_guiyan_turn") == turn_stamp(state)
                }
                _ => false,
            }
        }
        "txhj_xiefeng" => match event {
            EventData::TurnEndWindow { player, after: true } => {
                *player == owner
                    && is_unblocked_l11_owner(state, ctx.catalog, owner)
                    && !he_cards(state, owner).is_empty()
            }
            EventData::TurnEndWindow { after: false, .. } => {
                state.player(owner).alive && flag(state, owner, "l11_xiefeng_stacks") > 0
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != DamageNature::Thunder
                    && state.player(owner).alive
                    && flag(state, owner, "l11_xiefeng_stacks") > 0
            }
            _ => false,
        },
        "txhj_huangjin" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.targets.contains(&owner)
                    && is_unblocked_l11_owner(state, ctx.catalog, owner)
                    && u.effective_kind_in(state, ctx.catalog).is_sha_like()
        ),
        "txhj_juhun" => {
            if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner) {
                return false;
            }
            match event {
                EventData::TurnEndWindow { player, after: true } => *player == owner,
                EventData::TurnBegin { player } => {
                    let mask = flag(state, owner, "l11_juhun_dead_mask");
                    *player == owner
                        && (0..state.players.len() as PlayerId)
                            .any(|p| !state.player(p).alive && (mask & (1u32 << p)) != 0)
                }
                _ => false,
            }
        }
        "txhj_baolian" => {
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner)
        }
        "tx_modao" => {
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::Phase { player, phase: Phase::Prepare, begin: true } if *player == owner)
        }
        "txhj_beiming" => {
            frame.is_none_or(|f| f.stage == Stage::BeforeTriggers)
                && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Death { player, killer: Some(k) }
                        if *player == owner && *k != owner && state.player(*k).alive && !state.player(*k).hand.is_empty()
                )
        }
        "txhj_wangzu" => {
            frame.is_none_or(|f| f.stage == Stage::BeforeTriggers)
                && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Death { player, .. }
                        if *player == owner && state.turn.current != owner && state.turn.phase == Phase::Play && state.player(state.turn.current).alive
                )
        }
        "txhj_tunshi" => {
            let own_h = state.player(owner).hand.len();
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::TurnBegin { player } if *player == owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.is_friend_of(ctx.catalog, owner, p) && state.player(p).hand.len() > own_h)
        }
        "txhj_chuanyun" => {
            let own_hp = state.player(owner).hp;
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::TurnEndWindow { player, after: false } if *player == owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.is_friend_of(ctx.catalog, owner, p) && state.player(p).hp > own_hp)
        }
        "txhj_guihuo" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner)
                && !alive_others(state, owner).is_empty()
        }
        "txhj_luolei" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::Phase { player, phase: Phase::Prepare, begin: true } if *player == owner)
                && !alive_others(state, owner).is_empty()
        }
        "txhj_guimen" => matches!(
            event,
            EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                    && cards.iter().any(|&c| ctx.catalog.resolved_card_suit(state, c) == Suit::Spade)
        ),
        "txhj_zhouzu" => {
            state.player(owner).alive
                && matches!(event, EventData::TurnEndWindow { after: false, .. })
                && flag(state, owner, "l11_mahou_zhouzu_rem") > 0
        }
        "txhj_zhoufa" => {
            state.player(owner).alive
                && matches!(event, EventData::TurnEndWindow { after: false, .. })
                && flag(state, owner, "l11_mahou_zhoufa_rem") > 0
        }
        "txhj_renfang" => {
            state.player(owner).alive
                && matches!(event, EventData::TurnEndWindow { after: false, .. })
                && flag(state, owner, "l11_mahou_renfang_rem") > 0
        }
        "txhj_dizhou" => match event {
            EventData::TurnBegin { player } | EventData::TurnEndWindow { player, after: false } => {
                *player == owner
                    && is_unblocked_l11_owner(state, ctx.catalog, owner)
                    && !he_cards(state, owner).is_empty()
            }
            EventData::CardUsed(u) => {
                if u.cancelled
                    || u.player == owner
                    || !is_unblocked_l11_owner(state, ctx.catalog, owner)
                    || state.is_friend_of(ctx.catalog, owner, u.player)
                {
                    return false;
                }
                let Some(card) = u.card else {
                    return false;
                };
                let suit = ctx.catalog.effective_card_suit(state, u.player, card);
                state
                    .vm_aux
                    .piles
                    .get(&(owner, "txhj_dizhou".to_string()))
                    .is_some_and(|pile| {
                        pile.iter()
                            .any(|&pc| ctx.catalog.resolved_card_suit(state, pc) == suit)
                    })
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && is_unblocked_l11_owner(state, ctx.catalog, owner)
                    && state
                        .vm_aux
                        .piles
                        .get(&(owner, "txhj_dizhou".to_string()))
                        .is_some_and(|pile| !pile.is_empty())
            }
            _ => false,
        },
        "txhj_zhoulei" => match event {
            EventData::CardsDrawn { player, .. } => {
                if *player == owner || !is_locked_unblocked_l11_owner(state, ctx.catalog, owner) {
                    return false;
                }
                let mark = flag(state, owner, "l11_zhoulei_marks").min(5);
                let target_h = state.player(*player).hand.len() as i32;
                (target_h - mark as i32).abs() >= (5 - mark as i32)
            }
            _ => false,
        },
        "txqyyouji" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player == owner
                    && is_unblocked_l11_owner(state, ctx.catalog, owner)
                    && is_youji_eligible_use(ctx.catalog, u)
        ),
        _ => false,
    }
}
