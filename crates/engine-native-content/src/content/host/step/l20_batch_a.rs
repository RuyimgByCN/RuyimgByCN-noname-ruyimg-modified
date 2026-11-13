//! L20 标记补充 Batch A（8 个国战/修改技能 + 41 个纯标记/UI/音效桩，`标记补充/extension.js`）。
//!
//! 覆盖项：
//! 1. `wengua`（SP 蔡文姬·问卦，`6141`）
//! 2. `wengua5`（问卦全局副技能，`6145`）
//! 3. `diaohulishan`（调虎离山状态，`1432`）
//! 4. `_guozhan_marks`（国战标记整合发动，`1523`）
//! 5. `xianqu_mark`（先驱标记主动子技，`1651`）
//! 6. `_zhulianbihe_mark_tao`（珠联璧合当桃回复，`1747`）
//! 7. `_yinyang_mark_add`（阴阳鱼弃牌阶段手牌上限 +2，`1802`）
//! 8. `_aozhan_judge`（鏖战条件判定，`2152`）
//! 9. 41 个纯标记/UI/Spine/音效桩（`zhulianbihe_mark`、`yinyang_mark`、`yexinjia_mark` 及 38 个 `ui_hooks`）

use super::super::*;
use crate::catalog::{QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l20_batch_a_asks.rs"]
mod l20_batch_a_asks;
pub(in crate::content::host) use l20_batch_a_asks::ask_l20_batch_a;
use l20_batch_a_asks::*;

pub(super) const ASK_WENGUA_CHOOSE_CARD: u32 = 10;
pub(super) const ASK_WENGUA_CHOOSE_SIDE: u32 = 11;
pub(super) const ASK_WENGUA5_GIVE_CARD: u32 = 20;
pub(super) const ASK_WENGUA5_CHOOSE_SIDE: u32 = 21;
pub(super) const ASK_GUOZHAN_CHOOSE_MARK: u32 = 40;
pub(super) const ASK_XIANQU_CHOOSE_TARGET: u32 = 41;
pub(super) const ASK_XIANQU_CHOOSE_OPTION: u32 = 42;
pub(super) const ASK_ZHULIANBIHE_TAO_CONFIRM: u32 = 50;
pub(super) const ASK_YINYANG_ADD_CONFIRM: u32 = 60;

pub(in crate::content::host) fn trigger_meta_l20_batch_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "wengua" | "_guozhan_marks" | "xianqu_mark" => None,
        "zhulianbihe_mark" | "yinyang_mark" | "yexinjia_mark" => None,
        "wengua5" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnEnded, EventKind::TurnPhaseAfter],
        )),
        "diaohulishan" => Some(TriggerMeta {
            event: EventKind::DamageBefore,
            timing: Timing::First,
            priority: 10,
            order_selectable: false,
            optional: false,
            extra_events: vec![
                EventKind::Damage,
                EventKind::LoseHp,
                EventKind::Recover,
                EventKind::PhaseEnd,
                EventKind::TurnEnded,
                EventKind::TurnPhaseAfter,
            ],
        }),
        "_zhulianbihe_mark_tao" => Some(meta(EventKind::Dying, Vec::new())),
        "_yinyang_mark_add" => Some(meta(
            EventKind::PhaseBegin,
            vec![
                EventKind::PhaseEnd,
                EventKind::TurnEnded,
                EventKind::TurnPhaseAfter,
            ],
        )),
        "_aozhan_judge" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin])),
        "_xigua_feiyanbahu_"
        | "_guozhanChooseControlCHG"
        | "_guozhanhalfchghp"
        | "_binglintu"
        | "_dysp"
        | "_B_gameloging_guagua" => {
            Some(meta(EventKind::GameStart, vec![EventKind::TurnBegin]))
        }
        "_player_game_guagua" | "_T_phase_guagua" | "_player_phase_guagua" => {
            Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin]))
        }
        "_guozhan_quanjubiaoji_guagua" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnBegin, EventKind::SkillInvocation],
        )),
        "_guozhan_diebiaoji_" | "_shoushajisha_" => Some(meta(EventKind::Death, Vec::new())),
        "_shoushajisha_delete_" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnEnded, EventKind::TurnPhaseAfter],
        )),
        "_guozhan_duiweibiaoji_gua_" | "_guozhan_duiweibiaoji_gua_gua_" => Some(meta(
            EventKind::PhaseEnd,
            vec![EventKind::Death, EventKind::PhaseBegin],
        )),
        "_shiqihujia_"
        | "_diankuangtulu_gua"
        | "_wanjunqushou_gua"
        | "_player_baojiA"
        | "_player_baojiB"
        | "_player_baojiC"
        | "_player_baojiD"
        | "_player_baojiE"
        | "_player_baojiF"
        | "_player_baojiG"
        | "_player_baojiH"
        | "_player_daojianfu_xxgg_fuzi_" => {
            Some(meta(EventKind::DamageBefore, vec![EventKind::Damage]))
        }
        "_mianshang_" => Some(meta(
            EventKind::Damage,
            vec![EventKind::DamageZero, EventKind::DamageCancelled],
        )),
        "_o_o_recovertrigger" | "_o_o_miaoshouhuichun" | "_o_o_yishugaochao" => {
            Some(meta(EventKind::Recover, Vec::new()))
        }
        "_losehp_" => Some(meta(EventKind::LoseHp, Vec::new())),
        "_jiubuff_" | "_shanghaipaishiyongtexiao_" | "_shiyongkapaitexiao_xi_gua_" => {
            Some(meta(EventKind::CardUsed, Vec::new()))
        }
        "_jiubuff_xiaoshi_" => Some(meta(
            EventKind::CardUsed,
            vec![
                EventKind::Death,
                EventKind::PhaseEnd,
                EventKind::TurnEnded,
                EventKind::TurnPhaseAfter,
            ],
        )),
        "_shiyongkapaitexiao_" => {
            Some(meta(EventKind::CardUsed, vec![EventKind::CardResponded]))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l20_batch_a(name: &str) -> SkillLimits {
    match name {
        "wengua" | "wengua5" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_l20_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive && skill.name != "_guozhan_diebiaoji_" {
        return false;
    }
    match skill.name.as_str() {
        "wengua5" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player != owner,
            EventData::TurnEnded { player }
            | EventData::TurnEndWindow {
                player,
                after: true,
            } => flag(ctx.state, *player, "wengua2_active") > 0,
            _ => false,
        },
        "diaohulishan" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && (d.target == owner || flag(ctx.state, d.target, "diaohulishan_active") > 0)
            }
            EventData::LoseHp { player, amount } => {
                *amount > 0
                    && (*player == owner || flag(ctx.state, *player, "diaohulishan_active") > 0)
            }
            EventData::Recover { player, amount, .. } => {
                *amount > 0
                    && (*player == owner || flag(ctx.state, *player, "diaohulishan_active") > 0)
            }
            EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            }
            | EventData::TurnEnded { .. }
            | EventData::TurnEndWindow { after: true, .. } => true,
            _ => false,
        },
        "_zhulianbihe_mark_tao" => match event {
            EventData::Dying { player, .. } => {
                ctx.state.player(*player).alive
                    && ctx.state.player(*player).hp <= 0
                    && !ctx.catalog.has_diaohulishan(ctx.state, *player)
                    && !ctx.catalog.has_diaohulishan(ctx.state, owner)
                    && (has_gz_mark(ctx.state, ctx.catalog, owner, "zhulianbihe_mark")
                        || has_gz_mark(ctx.state, ctx.catalog, owner, "yexinjia_mark"))
            }
            _ => false,
        },
        "_yinyang_mark_add" => match event {
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: true,
            } => {
                *player == owner
                    && (has_gz_mark(ctx.state, ctx.catalog, owner, "yinyang_mark")
                        || has_gz_mark(ctx.state, ctx.catalog, owner, "yexinjia_mark"))
            }
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            }
            | EventData::TurnEnded { player }
            | EventData::TurnEndWindow {
                player,
                after: true,
            } => *player == owner && flag(ctx.state, owner, "yinyang_hand_bonus") > 0,
            _ => false,
        },
        "_aozhan_judge" => match event {
            EventData::TurnBegin { .. }
            | EventData::Phase {
                phase: Phase::Prepare,
                begin: true,
                ..
            } => aozhan_condition_met(ctx.state, ctx.catalog, owner),
            _ => false,
        },
        "_xigua_feiyanbahu_"
        | "_guozhanChooseControlCHG"
        | "_guozhanhalfchghp"
        | "_binglintu"
        | "_dysp"
        | "_B_gameloging_guagua" => matches!(
            event,
            EventData::Phase { begin: true, .. } | EventData::TurnBegin { .. }
        ),
        "_player_game_guagua" | "_T_phase_guagua" | "_player_phase_guagua" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player } => *player == owner,
            _ => false,
        },
        "_guozhan_quanjubiaoji_guagua" => match event {
            EventData::Phase {
                player,
                begin: true,
                ..
            }
            | EventData::TurnBegin { player } => {
                *player == owner
                    && (has_gz_mark(ctx.state, ctx.catalog, owner, "xianqu_mark")
                        || has_gz_mark(ctx.state, ctx.catalog, owner, "zhulianbihe_mark")
                        || has_gz_mark(ctx.state, ctx.catalog, owner, "yinyang_mark")
                        || has_gz_mark(ctx.state, ctx.catalog, owner, "yexinjia_mark"))
            }
            _ => false,
        },
        "_guozhan_diebiaoji_" => {
            matches!(event, EventData::Death { player, .. } if *player == owner)
        }
        "_shoushajisha_" => {
            matches!(event, EventData::Death { killer: Some(k), .. } if *k == owner)
        }
        "_shoushajisha_delete_" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            }
            | EventData::TurnEnded { player }
            | EventData::TurnEndWindow {
                player,
                after: true,
            } => *player == owner,
            _ => false,
        },
        "_guozhan_duiweibiaoji_gua_" | "_guozhan_duiweibiaoji_gua_gua_" => matches!(
            event,
            EventData::Phase {
                phase: Phase::Prepare,
                ..
            } | EventData::Death { .. }
        ),
        "_shiqihujia_" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && (ctx.state.player(owner).hujia > 0 || flag(ctx.state, owner, "hujia") > 0)
            }
            _ => false,
        },
        "_diankuangtulu_gua" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner) && d.target != owner && !d.prevented && d.amount == 3
            }
            _ => false,
        },
        "_wanjunqushou_gua" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner) && d.target != owner && !d.prevented && d.amount >= 4
            }
            _ => false,
        },
        "_player_baojiA" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && (d.amount == 1 || d.amount == 2))
        }
        "_player_baojiB" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && (d.amount == 2 || d.amount == 3))
        }
        "_player_baojiC" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && (d.amount == 3 || d.amount == 4))
        }
        "_player_baojiD" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && (d.amount == 4 || d.amount == 5))
        }
        "_player_baojiE" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && (d.amount == 5 || d.amount == 6))
        }
        "_player_baojiF" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && d.amount >= 6)
        }
        "_player_baojiG" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && d.amount == 8)
        }
        "_player_baojiH" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && d.amount >= 9)
        }
        "_player_daojianfu_xxgg_fuzi_" => {
            matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0 && d.source.is_some_and(|s| s != owner))
        }
        "_mianshang_" => {
            matches!(event, EventData::Damage(d) if d.target == owner && (d.prevented || d.amount <= 0))
        }
        "_o_o_recovertrigger" | "_o_o_yishugaochao" => {
            matches!(event, EventData::Recover { player, amount, .. } if *player == owner && *amount > 0)
        }
        "_o_o_miaoshouhuichun" => {
            matches!(event, EventData::Recover { player, amount, .. } if *amount > 0 && (*player == owner || ctx.state.turn.current == owner))
        }
        "_losehp_" => {
            matches!(event, EventData::LoseHp { player, amount } if *player == owner && *amount > 0)
        }
        "_jiubuff_" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && !ctx.state.player(owner).dying
                    && matches!(
                        u.effective_kind(ctx.catalog),
                        crate::catalog::CardKind::Jiu | crate::catalog::CardKind::Tianxianjiu
                    )
            }
            _ => false,
        },
        "_jiubuff_xiaoshi_" => match event {
            EventData::CardUsed(u) => {
                u.player == owner && u.effective_kind(ctx.catalog).is_sha_like()
            }
            EventData::Death { player, .. } => *player == owner,
            EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            }
            | EventData::TurnEnded { .. }
            | EventData::TurnEndWindow { after: true, .. } => true,
            _ => false,
        },
        "_shiyongkapaitexiao_" => match event {
            EventData::CardUsed(u) => u.player == owner,
            EventData::CardResponded { player, .. } => *player == owner,
            _ => false,
        },
        "_shiyongkapaitexiao_xi_gua_" => {
            matches!(event, EventData::CardUsed(u) if u.targets.contains(&owner))
        }
        "_shanghaipaishiyongtexiao_" => match event {
            EventData::CardUsed(u) => {
                let k = u.effective_kind(ctx.catalog);
                u.player == owner
                    && (k.is_sha_like()
                        || matches!(
                            k,
                            crate::catalog::CardKind::Duel
                                | crate::catalog::CardKind::SavageAttack
                                | crate::catalog::CardKind::ArcheryAttack
                                | crate::catalog::CardKind::ShouLiJian
                                | crate::catalog::CardKind::Linghunzhihuo
                        ))
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn activate_l20_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive
        || ctx.state.turn.current != owner
        || ctx.state.turn.phase != Phase::Play
        || ctx.catalog.has_diaohulishan(ctx.state, owner)
    {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "wengua" => !hand_and_equips(ctx.state, owner).is_empty(),
        "wengua5" => {
            !ctx.state.player(owner).hand.is_empty()
                && find_wengua_owner(ctx.state, ctx.catalog, owner).is_some()
        }
        "_guozhan_marks" => !guozhan_mark_options(ctx.state, ctx.catalog, owner).is_empty(),
        "xianqu_mark" => {
            has_gz_mark(ctx.state, ctx.catalog, owner, "xianqu_mark")
                || has_gz_mark(ctx.state, ctx.catalog, owner, "yexinjia_mark")
        }
        "_zhulianbihe_mark_tao" => {
            ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                && (has_gz_mark(ctx.state, ctx.catalog, owner, "zhulianbihe_mark")
                    || has_gz_mark(ctx.state, ctx.catalog, owner, "yexinjia_mark"))
        }
        _ => false,
    };
    if !can_activate {
        return None;
    }
    let target = if skill.name == "wengua5" {
        find_wengua_owner(ctx.state, ctx.catalog, owner)
    } else {
        None
    };
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L20MarkBatchA,
        owner,
        target,
        card: Some(100),
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l20_batch_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = Some(0u32);
    match event {
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            target = Some(*player);
            let code = match phase {
                Phase::Prepare => 1u32,
                Phase::Judge => 2,
                Phase::Draw => 3,
                Phase::Play => 4,
                Phase::Discard => 5,
                Phase::End => 6,
            };
            card = Some(if *begin { code } else { code + 10 });
        }
        EventData::TurnBegin { player } => {
            target = Some(*player);
            card = Some(1);
        }
        EventData::TurnEnded { player } | EventData::TurnEndWindow { player, .. } => {
            target = Some(*player);
            card = Some(90);
        }
        EventData::Dying { player, .. } => {
            target = Some(*player);
            card = Some(80);
        }
        EventData::Damage(d) => {
            target = Some(d.target);
            card = Some(70);
        }
        EventData::LoseHp { player, .. } => {
            target = Some(*player);
            card = Some(71);
        }
        EventData::Recover { player, .. } => {
            target = Some(*player);
            card = Some(72);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L20MarkBatchA,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l20_batch_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "wengua" => step_wengua(m, ctx),
        "wengua5" => step_wengua5(m, ctx),
        "diaohulishan" => step_diaohulishan(m, ctx),
        "_guozhan_marks" => step_guozhan_marks(m, ctx),
        "xianqu_mark" => step_xianqu_mark(m, ctx),
        "_zhulianbihe_mark_tao" => step_zhulianbihe_tao(m, ctx),
        "_yinyang_mark_add" => step_yinyang_add(m, ctx),
        "_aozhan_judge" => step_aozhan_judge(m, ctx),
        other => {
            if other == "_shoushajisha_" {
                let c = flag(ctx.core, m.owner, "_shoushajisha_count");
                set_flag(ctx.core, m.owner, "_shoushajisha_count", c + 1);
            } else if other == "_shoushajisha_delete_" {
                set_flag(ctx.core, m.owner, "_shoushajisha_count", 0);
            }
            let key = format!("l20_ui_hook:{other}");
            set_flag(ctx.core, m.owner, &key, 1);
            set_flag(ctx.core, m.owner, "l20_ui_hook_invoked", 1);
            Flow::Done
        }
    }
}

fn step_wengua(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(m, ctx, ASK_WENGUA_CHOOSE_CARD);
    }
    match m.card {
        Some(ASK_WENGUA_CHOOSE_CARD) => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "wengua");
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, ASK_WENGUA_CHOOSE_SIDE)
        }
        Some(ASK_WENGUA_CHOOSE_SIDE) => {
            let Some(ChoiceOption::Tag(side)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "wengua");
                return Flow::Done;
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            if !take_card_from_player(ctx, owner, cid) {
                return Flow::Done;
            }
            if side == 0 {
                ctx.core.deck.push(cid);
                let _ = draw_from_bottom(ctx, owner);
                set_flag(ctx.core, owner, "wengua_side", 0);
            } else {
                ctx.core.deck.insert(0, cid);
                let _ = draw_from_top_n(ctx, owner, 1);
                set_flag(ctx.core, owner, "wengua_side", 1);
            }
            set_flag(ctx.core, owner, "wengua_resolved", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_wengua5(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage == HostStage::Act && m.card != Some(100) {
        let turn_p = m.target.unwrap_or(ctx.core.turn.current);
        if m.card == Some(90) || m.card == Some(16) {
            set_flag(ctx.core, turn_p, "wengua2_active", 0);
            set_flag(ctx.core, turn_p, "wengua_owner_seat", 0);
        } else if turn_p != m.owner {
            set_flag(ctx.core, turn_p, "wengua2_active", 1);
            set_flag(ctx.core, turn_p, "wengua_owner_seat", u32::from(m.owner) + 1);
            if let Some(def) = ctx.catalog.skill_by_name("wengua5") {
                ctx.core
                    .skills
                    .entry((turn_p, def))
                    .or_insert(crate::state::SkillInstance {
                        owner: turn_p,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: true,
                        rounds: 0,
                    });
            }
        }
        return Flow::Done;
    }
    let giver = m.owner;
    let Some(wg_owner) = m
        .target
        .or_else(|| find_wengua_owner(ctx.core, ctx.catalog, giver))
    else {
        refund_once_used(ctx, giver, "wengua5");
        return Flow::Done;
    };
    m.target = Some(wg_owner);
    if m.stage == HostStage::Act {
        return wait_ask(m, ctx, ASK_WENGUA5_GIVE_CARD);
    }
    match m.card {
        Some(ASK_WENGUA5_GIVE_CARD) => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, giver, "wengua5");
                return Flow::Done;
            };
            if !give_hand_card(ctx, giver, wg_owner, cid) {
                refund_once_used(ctx, giver, "wengua5");
                return Flow::Done;
            }
            m.cards = vec![cid];
            set_flag(ctx.core, giver, "wengua5_given_card", cid + 1);
            wait_ask(m, ctx, ASK_WENGUA5_CHOOSE_SIDE)
        }
        Some(ASK_WENGUA5_CHOOSE_SIDE) => {
            let side = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(tag)) => tag,
                _ => 2,
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            match side {
                0 => {
                    if take_card_from_player(ctx, wg_owner, cid) {
                        ctx.core.deck.push(cid);
                    }
                    let _ = draw_from_bottom(ctx, wg_owner);
                    let _ = draw_from_bottom(ctx, giver);
                    set_flag(ctx.core, wg_owner, "wengua5_side", 0);
                    set_flag(ctx.core, giver, "wengua5_side", 0);
                }
                1 => {
                    if take_card_from_player(ctx, wg_owner, cid) {
                        ctx.core.deck.insert(0, cid);
                    }
                    let _ = draw_from_top_n(ctx, wg_owner, 1);
                    let _ = draw_from_top_n(ctx, giver, 1);
                    set_flag(ctx.core, wg_owner, "wengua5_side", 1);
                    set_flag(ctx.core, giver, "wengua5_side", 1);
                }
                _ => {
                    set_flag(ctx.core, wg_owner, "wengua5_side", 2);
                    set_flag(ctx.core, giver, "wengua5_side", 2);
                }
            }
            set_flag(ctx.core, wg_owner, "wengua5_resolved", 1);
            set_flag(ctx.core, giver, "wengua5_resolved", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_diaohulishan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let t = m.target.unwrap_or(m.owner);
    match m.card {
        Some(70) => {
            if let Some(f) = ctx.res.stack.last_mut() {
                if matches!(f.data, EventData::Damage(_)) {
                    f.damage_mut().prevented = true;
                    f.damage_mut().amount = 0;
                }
            }
            set_flag(ctx.core, t, "diaohulishan_prevented_damage", 1);
        }
        Some(71) => set_flag(ctx.core, t, "diaohulishan_prevented_lose_hp", 1),
        Some(72) => set_flag(ctx.core, t, "diaohulishan_prevented_recover", 1),
        Some(90) | Some(16) => {
            let dhls_def = ctx.catalog.skill_by_name("diaohulishan");
            for idx in 0..ctx.core.players.len() {
                let p = idx as PlayerId;
                let had_flag = flag(ctx.core, p, "diaohulishan_active") > 0;
                let had_skill = dhls_def.is_some_and(|d| ctx.core.skills.contains_key(&(p, d)));
                if had_flag || had_skill {
                    set_flag(ctx.core, p, "diaohulishan_active", 0);
                    set_flag(ctx.core, p, "l20_diaohulishan_exiled", 0);
                    ctx.core.player_mut(p).skip_discard_phase = false;
                    ctx.core.player_mut(p).skip_next_draw = false;
                    set_flag(ctx.core, p, "diaohulishan_cleared", 1);
                    if let Some(d) = dhls_def {
                        ctx.core.skills.remove(&(p, d));
                    }
                }
            }
        }
        _ => {}
    }
    Flow::Done
}

fn apply_xianqu_option(core: &mut GameStateCore, owner: PlayerId, tag: u8) {
    match tag {
        1 => set_flag(core, owner, "xianqu_viewed_main", 1),
        2 => set_flag(core, owner, "xianqu_viewed_deputy", 1),
        _ => {}
    }
    set_flag(core, owner, "xianqu_option", u32::from(tag) + 1);
}

fn resolve_xianqu_target(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    m.target = Some(t);
    set_flag(ctx.core, owner, "xianqu_viewed", 1);
    set_flag(ctx.core, owner, "xianqu_target", u32::from(t) + 1);
    set_flag(ctx.core, t, "xianqu_inspected_by", u32::from(owner) + 1);
    let opts = xianqu_target_options(ctx.core, t);
    if opts.len() > 1 {
        return wait_ask(m, ctx, ASK_XIANQU_CHOOSE_OPTION);
    }
    if let Some(ChoiceOption::Tag(tag)) = opts.first().copied() {
        apply_xianqu_option(ctx.core, owner, tag);
    }
    Flow::Done
}

fn execute_xianqu_effect(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    if !consume_gz_mark_or_yexinjia(ctx.core, ctx.catalog, owner, "xianqu_mark") {
        return Flow::Done;
    }
    let cur_hand = ctx.core.player(owner).hand.len();
    if cur_hand < 4 {
        let _ = draw_from_top_n(ctx, owner, (4 - cur_hand) as u32);
    }
    set_flag(ctx.core, owner, "xianqu_viewed", 1);
    if !xianqu_valid_targets(ctx.core, owner).is_empty() {
        wait_ask(m, ctx, ASK_XIANQU_CHOOSE_TARGET)
    } else {
        Flow::Done
    }
}

fn step_guozhan_marks(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(m, ctx, ASK_GUOZHAN_CHOOSE_MARK);
    }
    match m.card {
        Some(ASK_GUOZHAN_CHOOSE_MARK) => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            match tag {
                0 => execute_xianqu_effect(m, ctx, owner),
                1 => {
                    if consume_gz_mark_or_yexinjia(
                        ctx.core,
                        ctx.catalog,
                        owner,
                        "zhulianbihe_mark",
                    ) {
                        let _ = draw_from_top_n(ctx, owner, 2);
                        set_flag(ctx.core, owner, "zhulianbihe_drew", 2);
                    }
                    Flow::Done
                }
                2 => {
                    if consume_gz_mark_or_yexinjia(ctx.core, ctx.catalog, owner, "yinyang_mark") {
                        let _ = draw_from_top_n(ctx, owner, 1);
                        set_flag(ctx.core, owner, "yinyang_drew", 1);
                    }
                    Flow::Done
                }
                3 => {
                    if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
                        && consume_gz_mark_or_yexinjia(
                            ctx.core,
                            ctx.catalog,
                            owner,
                            "zhulianbihe_mark",
                        )
                    {
                        ctx.recover(owner, 1, None);
                        set_flag(ctx.core, owner, "_zhulianbihe_tao_used", 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        Some(ASK_XIANQU_CHOOSE_TARGET) => resolve_xianqu_target(m, ctx, owner),
        Some(ASK_XIANQU_CHOOSE_OPTION) => {
            if let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() {
                apply_xianqu_option(ctx.core, owner, tag);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xianqu_mark(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return execute_xianqu_effect(m, ctx, owner);
    }
    match m.card {
        Some(ASK_XIANQU_CHOOSE_TARGET) => resolve_xianqu_target(m, ctx, owner),
        Some(ASK_XIANQU_CHOOSE_OPTION) => {
            if let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() {
                apply_xianqu_option(ctx.core, owner, tag);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhulianbihe_tao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(m, ctx, ASK_ZHULIANBIHE_TAO_CONFIRM);
    }
    if m.card == Some(ASK_ZHULIANBIHE_TAO_CONFIRM)
        && matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm))
        && consume_gz_mark_or_yexinjia(ctx.core, ctx.catalog, owner, "zhulianbihe_mark")
    {
        let rec_target = m.target.unwrap_or(owner);
        ctx.recover(rec_target, 1, None);
        set_flag(ctx.core, owner, "_zhulianbihe_tao_used", 1);
        set_flag(ctx.core, owner, "_zhulianbihe_tao_target", u32::from(rec_target) + 1);
    }
    Flow::Done
}

fn step_yinyang_add(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        if m.card == Some(90) || m.card == Some(16) {
            set_flag(ctx.core, owner, "yinyang_hand_bonus", 0);
            return Flow::Done;
        }
        return wait_ask(m, ctx, ASK_YINYANG_ADD_CONFIRM);
    }
    if m.card == Some(ASK_YINYANG_ADD_CONFIRM)
        && matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm))
        && consume_gz_mark_or_yexinjia(ctx.core, ctx.catalog, owner, "yinyang_mark")
    {
        let cur = flag(ctx.core, owner, "yinyang_hand_bonus");
        set_flag(ctx.core, owner, "yinyang_hand_bonus", cur + 2);
        set_flag(ctx.core, owner, "_yinyang_mark_add_used", 1);
    }
    Flow::Done
}

fn step_aozhan_judge(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if aozhan_condition_met(ctx.core, ctx.catalog, owner) {
        for idx in 0..ctx.core.players.len() {
            set_flag(ctx.core, idx as PlayerId, "aozhan_active", 1);
        }
    }
    Flow::Done
}
