//! U01 Sub-batch 2 (`3D精选` 21 技) 上游等价修复模块。

use super::super::*;
use super::u01_batch_a::{
    flag, hand_options, he_cards, he_options, set_flag, ASK_KEY,
};
use super::u01_fixes_1::{
    can_use_sha_play_phase, effective_is_red, effective_suit_bit, finish_respond_park_pub,
    is_black, is_delayed, is_scope4_used, is_sha_target_blocked, is_target_protected,
    jiedao_victims_for, mark_scope4_used, u01_discard_batch, u01_give_card_ex, u01_remove_owned_ex,
    virtual_kind_can_use_play, virtual_kind_needs_target_prompt, virtual_legal_targets,
    zhishu_valid_dests,
};
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

mod asks_b;
mod asks_c;
mod helpers;
mod routing;
mod step_b1;
mod step_b2;
mod step_c1;
mod step_c2;

use helpers::*;
pub(crate) use routing::*;

pub(crate) fn is_u01_fix_2_batch_b(name: &str) -> bool {
    matches!(
        name,
        "dddzhengjun"
            | "dddxianxi"
            | "dddlingyong"
            | "dddxuxiao"
            | "dddxuyu"
            | "dddshijian"
            | "dddxuyu_old"
            | "dddshijian_old"
            | "dddtaisi"
            | "dddquche"
            | "dddqianlong"
            | "ddddongcha"
            | "dddzhijie"
    )
}

pub(crate) fn is_u01_fix_2_batch_c(name: &str) -> bool {
    matches!(
        name,
        "dddqiahua"
            | "dddxunxun"
            | "dddfusi"
            | "dddfusi_global"
            | "dddtuoji"
            | "dddchashi"
            | "dddmiaoxing"
            | "dddfenji"
    )
}

pub(crate) fn is_u01_fix_2_name(name: &str) -> bool {
    is_u01_fix_2_batch_b(name) || is_u01_fix_2_batch_c(name)
}

pub(crate) fn runs_before_phase_begin_2(name: &str) -> bool {
    matches!(name, "dddxunxun" | "dddmiaoxing" | "dddfenji")
}

pub(crate) fn active_ask_u01_fix2(
    ask_id: u32,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let _ = stage;
    let (observer, spec) =
        build_ask_2(ask_id, owner, target, targets, card, cards, core, catalog)?;
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

pub(super) fn build_ask_2(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        2001..=2034 => {
            asks_b::build_ask_b_2(ask_id, owner, target, targets, card, cards, core, catalog)
        }
        2035..=2060 => {
            asks_c::build_ask_c_2(ask_id, owner, target, targets, card, cards, core, catalog)
        }
        _ => None,
    }
}

pub(crate) fn u01_phase_skipped_hook(ctx: &mut Ctx<'_>, _skipped_player: PlayerId) {
    let Some(def) = ctx.catalog.skill_by_name("dddlingyong") else {
        return;
    };
    let n = ctx.core.players.len() as PlayerId;
    let cur = ctx.core.turn.current;
    for off in (0..n).rev() {
        let owner = (cur + off) % n;
        if ctx.core.player(owner).alive
            && ctx.core.has_skill(owner, def)
            && !ctx.catalog.is_tieji_sealed(ctx.core, owner)
        {
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "dddlingyong".to_string(),
                    behavior: HostBehavior::U01DddBatchB,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
            });
        }
    }
}

fn push_intrinsic_park(
    ctx: &mut Ctx<'_>,
    def_key: &str,
    behavior: HostBehavior,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<u32>,
    targets: Vec<PlayerId>,
) {
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: def_key.to_string(),
            behavior,
            owner,
            target,
            card,
            cards: Vec::new(),
            targets,
            stage: HostStage::Act,
        }),
    });
}

pub(crate) fn u01_card_use_after_hook_2(ctx: &mut Ctx<'_>, u: &CardUseData) {
    let fid = flag(ctx.core, u.player, "last_popped_card_use_fid");
    if fid != 0 && flag(ctx.core, u.player, "dddfenji_active_fid") == fid {
        ctx.core
            .vm_aux
            .flags
            .remove(&(u.player, "dddfenji_active_fid".to_string()));
        let dmg = flag(ctx.core, u.player, &format!("dddfenji_dmg:{fid}"));
        set_flag(ctx.core, u.player, "dddfenji_result_dmg", 1 + dmg);
        let tcount = if u.effective_kind(ctx.catalog) == CardKind::Jiedao {
            usize::from(!u.targets.is_empty())
        } else {
            u.targets.len()
        };
        set_flag(
            ctx.core,
            u.player,
            "dddfenji_result_targets",
            1 + tcount as u32,
        );
    }

    if fid != 0 {
        let added = flag(ctx.core, u.player, &format!("dddxianxi_added:{fid}"));
        let damaged = flag(ctx.core, u.player, &format!("dddxianxi_dmg:{fid}"));
        let missed = added & !damaged;
        if missed != 0 && ctx.core.player(u.player).alive {
            let n = ctx.core.players.len() as PlayerId;
            let cur = ctx.core.turn.current;
            let mut missed_targets = Vec::new();
            for off in 0..n {
                let p = (cur + off) % n;
                if (missed & (1u32 << p)) != 0 && ctx.core.player(p).alive {
                    missed_targets.push(p);
                }
            }
            if !missed_targets.is_empty() {
                push_intrinsic_park(
                    ctx,
                    "dddxianxi",
                    HostBehavior::U01DddBatchB,
                    u.player,
                    None,
                    Some(10),
                    missed_targets,
                );
            }
        }
    }

    if flag(ctx.core, u.player, "dddzhijie_pending_draw") > 0
        && matches!(u.use_as, Some(CardKind::Shan | CardKind::Nullify))
    {
        let pending = flag(ctx.core, u.player, "dddzhijie_pending_draw");
        set_flag(ctx.core, u.player, "dddzhijie_pending_draw", pending - 1);
        ctx.start_draw(u.player, 2);
    }

    if let Some(tuoji_def) = ctx.catalog.skill_by_name("dddtuoji") {
        let fusi_src = fid
            .ne(&0)
            .then(|| flag(ctx.core, u.player, &format!("dddfusi_used_owner:{fid}")))
            .flatten_or_zero();
        let n = ctx.core.players.len() as PlayerId;
        let cur = ctx.core.turn.current;
        for off in (0..n).rev() {
            let owner = (cur + off) % n;
            let matches_owner = fusi_src == u32::from(owner) + 1;
            if matches_owner
                && ctx.core.player(owner).alive
                && ctx.core.has_skill(owner, tuoji_def)
                && !ctx.catalog.is_tieji_sealed(ctx.core, owner)
                && (ctx.core.player(owner).hand.is_empty() || dddfusi_all_shown(ctx.core, owner))
            {
                push_intrinsic_park(
                    ctx,
                    "dddtuoji",
                    HostBehavior::U01DddBatchC,
                    owner,
                    None,
                    None,
                    Vec::new(),
                );
            }
        }
    }

    let play_seq = if fid != 0 {
        flag(ctx.core, u.player, &format!("card_use_play_seq:{fid}"))
    } else {
        0
    };
    let play_idx = if fid != 0 {
        flag(ctx.core, u.player, &format!("card_use_play_idx:{fid}"))
    } else {
        0
    };
    let cur_seq = flag(ctx.core, 0, "play_phase_seq").max(1);

    if play_seq != 0 && play_seq == cur_seq && play_idx > 0 {
        let n = ctx.core.players.len() as PlayerId;
        let cur = ctx.core.turn.current;
        for off in (0..n).rev() {
            let owner = (cur + off) % n;
            if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
                continue;
            }
            if owner != u.player
                && play_idx == 2
                && ctx
                    .catalog
                    .skill_by_name("dddshijian")
                    .is_some_and(|d| ctx.core.has_skill(owner, d))
                && !he_cards(ctx.core, owner).is_empty()
            {
                push_intrinsic_park(
                    ctx,
                    "dddshijian",
                    HostBehavior::U01DddBatchB,
                    owner,
                    Some(u.player),
                    Some(1),
                    Vec::new(),
                );
            }
            if owner != u.player
                && ctx
                    .catalog
                    .skill_by_name("dddshijian_old")
                    .is_some_and(|d| ctx.core.has_skill(owner, d))
            {
                if play_idx == 3 && ctx.core.player(u.player).alive {
                    push_intrinsic_park(
                        ctx,
                        "dddshijian_old",
                        HostBehavior::U01DddBatchB,
                        owner,
                        Some(u.player),
                        Some(play_seq),
                        Vec::new(),
                    );
                } else if play_idx == 4 {
                    let armed_key = format!("dddshijian_old_armed:{play_seq}");
                    if flag(ctx.core, owner, &armed_key) == u32::from(u.player) + 1 {
                        ctx.core.vm_aux.flags.remove(&(owner, armed_key));
                        if !he_cards(ctx.core, u.player).is_empty() {
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "dddshijian_old",
                                owner,
                                presentation: None,
                            });
                            ctx.start_draw(owner, 2);
                        }
                    }
                }
            }
            if owner == u.player
                && ctx
                    .catalog
                    .skill_by_name("dddxuyu_old")
                    .is_some_and(|d| ctx.core.has_skill(owner, d))
            {
                if play_idx == 1 {
                    set_flag(ctx.core, owner, "dddxuyu_old_play_seq", play_seq);
                    push_intrinsic_park(
                        ctx,
                        "dddxuyu_old",
                        HostBehavior::U01DddBatchB,
                        owner,
                        None,
                        None,
                        Vec::new(),
                    );
                } else if play_idx == 3 {
                    let armed_key = format!("dddxuyu_old_armed:{play_seq}");
                    if flag(ctx.core, owner, &armed_key) == 1 {
                        ctx.core.vm_aux.flags.remove(&(owner, armed_key));
                        if !he_cards(ctx.core, owner).is_empty() {
                            push_intrinsic_park(
                                ctx,
                                "dddxuyu_old",
                                HostBehavior::U01DddBatchB,
                                owner,
                                Some(owner),
                                None,
                                Vec::new(),
                            );
                        }
                    }
                }
            }
        }
    }

    if ctx.core.player(u.player).alive {
        let pending_fid = flag(ctx.core, u.player, "dddxuyu_pending_fid");
        let trigger_discard = pending_fid != 0 && pending_fid != fid.max(1);
        let has_xuyu = ctx
            .catalog
            .skill_by_name("dddxuyu")
            .is_some_and(|d| ctx.core.has_skill(u.player, d))
            && !ctx.catalog.is_tieji_sealed(ctx.core, u.player);
        if has_xuyu {
            set_flag(ctx.core, u.player, "dddxuyu_next_fid", fid.max(1));
            push_intrinsic_park(
                ctx,
                "dddxuyu",
                HostBehavior::U01DddBatchB,
                u.player,
                None,
                None,
                Vec::new(),
            );
        }
        if trigger_discard {
            ctx.core
                .vm_aux
                .flags
                .remove(&(u.player, "dddxuyu_pending_fid".to_string()));
            if !he_cards(ctx.core, u.player).is_empty() {
                push_intrinsic_park(
                    ctx,
                    "dddxuyu",
                    HostBehavior::U01DddBatchB,
                    u.player,
                    Some(u.player),
                    None,
                    Vec::new(),
                );
            }
        }
    }
}

trait FlattenOrZero {
    fn flatten_or_zero(self) -> u32;
}
impl FlattenOrZero for Option<u32> {
    fn flatten_or_zero(self) -> u32 {
        self.unwrap_or(0)
    }
}

pub(crate) fn u01_respond_skill_options_2(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    if is_use
        && !catalog.is_tieji_sealed(core, player)
        && matches!(need, CardKind::Shan | CardKind::Nullify)
    {
        if let Some(def) = catalog.skill_by_name("dddzhijie") {
            if core.has_skill(player, def) && dddzhijie_can_respond(core, catalog, player, need) {
                out.push(ChoiceOption::Skill(def));
            }
        }
    }
    if is_use {
        if let Some(def) = catalog.skill_by_name("dddfusi_global") {
            if core.has_skill(player, def)
                && flag(core, player, "dddfusi_refused") == 0
                && flag(core, player, "dddfusi_window_refused") == 0
                && !dddfusi_respond_cards(core, catalog, player, need).is_empty()
            {
                out.push(ChoiceOption::Skill(def));
            }
        }
    }
}

pub(crate) fn u01_try_activate_respond_skill_2(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    fid: EventId,
    is_use: bool,
) -> Option<bool> {
    let name = ctx.catalog.skill_name(def);
    match name {
        "dddzhijie" => {
            if !is_use {
                return Some(false);
            }
            set_flag(ctx.core, player, "u01_respond_fid", fid);
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger { frame: fid },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "dddzhijie".to_string(),
                    behavior: HostBehavior::U01DddBatchB,
                    owner: player,
                    target: None,
                    card: Some(encode_respond_need(need)),
                    cards: Vec::new(),
                    targets: vec![u8::from(is_use)],
                    stage: HostStage::Act,
                }),
            });
            Some(false)
        }
        "dddfusi_global" => {
            if !is_use {
                return Some(false);
            }
            set_flag(ctx.core, player, "u01_respond_fid", fid);
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger { frame: fid },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "dddfusi_global".to_string(),
                    behavior: HostBehavior::U01DddBatchC,
                    owner: player,
                    target: None,
                    card: Some(encode_respond_need(need)),
                    cards: Vec::new(),
                    targets: vec![u8::from(is_use)],
                    stage: HostStage::Act,
                }),
            });
            Some(false)
        }
        _ => None,
    }
}
