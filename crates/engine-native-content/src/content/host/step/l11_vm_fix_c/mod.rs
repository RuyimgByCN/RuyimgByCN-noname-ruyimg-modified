//! Track `P3-E3`: native `HostBehavior::L11VmFixC` implementation of the `42` stripped
//! `taixu-upstream-liuli` skills (+ `7` subskills) against `noname-liuli` upstream JS.

mod active;
mod asks;
mod combat_triggers;
mod filter;
mod helpers;
mod hooks;
mod meta;
mod phase_triggers;

use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

pub(super) const ASK_CHONGZHEN_CONFIRM: u32 = 1;
pub(super) const ASK_CHONGZHEN_CARD: u32 = 2;
pub(super) const ASK_DUJIN_CONFIRM: u32 = 3;
pub(super) const ASK_GNJINFAN_CARDS: u32 = 4;
pub(super) const ASK_HUOJI_CARD: u32 = 5;
pub(super) const ASK_HUOJI_TARGET: u32 = 6;
pub(super) const ASK_JUECE_CONFIRM: u32 = 7;
pub(super) const ASK_LONGHUN_KIND: u32 = 8;
pub(super) const ASK_LONGHUN_CARDS: u32 = 9;
pub(super) const ASK_LONGHUN_TARGET: u32 = 10;
pub(super) const ASK_MIEJI_TARGET: u32 = 11;
pub(super) const ASK_NSFEIXIONG_TARGET: u32 = 12;
pub(super) const ASK_NSFEIXIONG_PINDIAN_OWNER: u32 = 13;
pub(super) const ASK_NSFEIXIONG_PINDIAN_TARGET: u32 = 14;
pub(super) const ASK_NSYICE_ORDER: u32 = 15;
pub(super) const ASK_NSYICE_TOP: u32 = 16;
pub(super) const ASK_NSYICE_DAMAGE_TARGET: u32 = 17;
pub(super) const ASK_NZRY_JIEYING_TARGET: u32 = 18;
pub(super) const ASK_QIAOSHUI_TARGET: u32 = 19;
pub(super) const ASK_QIAOSHUI_PINDIAN_OWNER: u32 = 20;
pub(super) const ASK_QIAOSHUI_PINDIAN_TARGET: u32 = 21;
pub(super) const ASK_QIAOSHUI3_ADD_TARGET: u32 = 22;
pub(super) const ASK_QIAOSHUI3_REMOVE_TARGET: u32 = 23;
pub(super) const ASK_QICE_TRICK: u32 = 24;
pub(super) const ASK_QICE_TARGETS: u32 = 25;
pub(super) const ASK_REJIZHI_CONFIRM: u32 = 26;
pub(super) const ASK_REJIZHI_DISCARD: u32 = 27;
pub(super) const ASK_RELIANJI_TARGETS: u32 = 28;
pub(super) const ASK_REQIANGXI_TARGET: u32 = 29;
pub(super) const ASK_REQIANGXI_WEAPON: u32 = 30;
pub(super) const ASK_RESHANXI_COST: u32 = 31;
pub(super) const ASK_RESHANXI_TARGET: u32 = 32;
pub(super) const ASK_RESHANXI_EXILE: u32 = 33;
pub(super) const ASK_RETIEJI_CONFIRM: u32 = 34;
pub(super) const ASK_RETIEJI_DISCARD: u32 = 35;
pub(super) const ASK_REZHENXING_CONFIRM: u32 = 36;
pub(super) const ASK_REZHENXING_PICK: u32 = 37;
pub(super) const ASK_REZHIHENG_CARDS: u32 = 38;
pub(super) const ASK_SHELIE_PICK: u32 = 39;
pub(super) const ASK_TWJIANGE_CARD: u32 = 40;
pub(super) const ASK_TWJIANGE_TARGET: u32 = 41;
pub(super) const ASK_TWKUJIAN_CARDS: u32 = 42;
pub(super) const ASK_TWKUJIAN_TARGET: u32 = 43;
pub(super) const ASK_TWKUJIAN_DISCARD: u32 = 44;
pub(super) const ASK_XINANJIAN_MODE: u32 = 45;
pub(super) const ASK_XINKUANGGU_MODE: u32 = 46;
pub(super) const ASK_XINNILUAN_CARD: u32 = 47;
pub(super) const ASK_XINNILUAN_DISCARD: u32 = 48;
pub(super) const ASK_XIONGYI_TARGETS: u32 = 49;
pub(super) const ASK_ZHUIKONG_CONFIRM: u32 = 50;
pub(super) const ASK_ZHUIKONG_PINDIAN_OWNER: u32 = 51;
pub(super) const ASK_ZHUIKONG_PINDIAN_TARGET: u32 = 52;
pub(super) const ASK_PINDIAN_YINGYANG_OWNER: u32 = 53;
pub(super) const ASK_PINDIAN_YINGYANG_TARGET: u32 = 54;

pub(in crate::content::host) fn limits_l11_vm_c(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn trigger_meta_l11_vm_c(
    name: &str,
    kind: EventKind,
) -> Option<TriggerMeta> {
    meta::trigger_meta_l11_vm_c(name, kind)
}

pub(in crate::content::host) fn has_phase_use_activation_l11_vm_c(name: &str) -> bool {
    active::has_phase_use_l11_vm_c(name)
}

pub(in crate::content::host) fn activate_l11_vm_c(
    skill: &HostSkillBehavior,
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<HostMachine> {
    active::activate_l11_vm_c(skill, state, catalog, owner)
}

pub(in crate::content::host) fn filter_l11_vm_c_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    filter::filter_l11_vm_c_with_frames(skill, ctx, frames, frame, owner, event)
}

pub(in crate::content::host) fn machine_l11_vm_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut m = HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11VmFixC,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    };
    match event {
        EventData::CardUsed(u) => {
            m.target = Some(u.player);
            m.card = u.card;
            m.targets = u.targets.clone();
            if let Some(k) = u.use_as {
                m.cards = vec![k as u32];
            }
        }
        EventData::CardResponded { card, player } => {
            m.target = Some(*player);
            m.card = Some(*card);
        }
        EventData::Damage(d) => {
            m.target = Some(d.target);
            m.card = d.card;
            m.targets = d.source.into_iter().collect();
            m.cards = vec![d.amount.max(0) as u32];
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            m.target = Some(*player);
            m.card = Some(if *begin {
                (*phase as u32) + 1
            } else {
                (*phase as u32) + 11
            });
        }
        EventData::TurnBegin { player } => {
            m.target = Some(*player);
        }
        EventData::CardsDiscarded { player, cards } => {
            m.target = Some(*player);
            m.cards = cards.clone();
        }
        EventData::CardsLost { player, cards, .. } => {
            m.target = Some(*player);
            m.cards = cards.clone();
        }
        _ => {}
    }
    m
}

pub(in crate::content::host) fn handle_l11_vm_c(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act
        && m.stage != HostStage::DrawSettle
        && m.stage != HostStage::ShuchuiTargetLoop
        && ctx.answer().is_empty()
    {
        let ask_id = helpers::flag(ctx.core, owner, "l11c_ask_id");
        if asks::has_custom_cancel(ask_id) {
            return dispatch_handle(m, ctx);
        }
        helpers::clear_ask(ctx, owner);
        if m.def_key.starts_with("respond:") {
            helpers::finish_respond_result(ctx, owner, false);
        }
        return Flow::Done;
    }
    dispatch_handle(m, ctx)
}

fn dispatch_handle(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.def_key.starts_with("respond:") {
        return active::step_respond(m, ctx);
    }
    match m.def_key.as_str() {
        "huoji" | "longhun" | "qice" | "relianji" | "reqiangxi" | "rezhiheng" | "twjiange"
        | "twkujian" | "xiongyi" => active::step_active(m, ctx),
        "anjian" | "baonu" | "chongzhen" | "mieji" | "qiaoshui3" | "ranshang" | "rejizhi"
        | "retieji" | "rezhenxing" | "shibei" | "twkujian_draw" | "xinanjian" | "xinkuanggu" => {
            combat_triggers::step_combat(m, ctx)
        }
        _ => phase_triggers::step_phase(m, ctx),
    }
}

pub(in crate::content::host) fn ask_l11_vm_c(
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<crate::snap::ExpectedAsk> {
    asks::ask_l11_vm_c(owner, target, targets, card, cards, core, catalog)
}

pub(in crate::content::host) fn respond_skill_options_l11_vm_c(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    active::respond_skill_options_l11_vm_c(core, catalog, player, need, is_use, out);
}

pub(in crate::content::host) fn try_activate_respond_skill_l11_vm_c(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    fid: EventId,
    is_use: bool,
) -> Option<bool> {
    active::try_activate_respond_skill_l11_vm_c(ctx, player, def, need, fid, is_use)
}

pub(in crate::content::host) fn fuli_dying_save_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    dying: PlayerId,
    saver: PlayerId,
) -> Vec<SkillDefId> {
    hooks::fuli_dying_save_skills(core, catalog, dying, saver)
}

pub(in crate::content::host) fn resolve_fuli_dying_save(
    ctx: &mut Ctx<'_>,
    dying: PlayerId,
    saver: PlayerId,
    def: SkillDefId,
) -> bool {
    hooks::resolve_fuli_dying_save(ctx, dying, saver, def)
}

pub(in crate::content::host) fn on_game_start_l11_vm_c(
    core: &mut GameStateCore,
    catalog: &Catalog,
) {
    hooks::on_game_start_l11_vm_c(core, catalog);
}

pub(in crate::content::host) fn on_skill_granted_l11_vm_c(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    def: SkillDefId,
) {
    hooks::on_skill_granted_l11_vm_c(core, catalog, player, def);
}

pub(in crate::content::host) fn on_skill_removed_l11_vm_c(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    hooks::on_skill_removed_l11_vm_c(core, catalog, owner, skill_name)
}

pub(in crate::content::host) fn on_card_use_after_l11_vm_c(
    ctx: &mut Ctx<'_>,
    fid: EventId,
    u: &crate::event::CardUseData,
) {
    hooks::on_card_use_after_l11_vm_c(ctx, fid, u);
}

pub(in crate::content::host) fn on_damage_end_l11_vm_c(
    ctx: &mut Ctx<'_>,
    d: &crate::event::DamageData,
) {
    hooks::on_damage_end_l11_vm_c(ctx, d);
}

pub(in crate::content::host) fn on_phase_end_l11_vm_c(ctx: &mut Ctx<'_>) {
    hooks::on_phase_end_l11_vm_c(ctx);
}

pub(in crate::content::host) fn on_turn_phase_after_l11_vm_c(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
) {
    hooks::on_turn_phase_after_l11_vm_c(ctx, player);
}

pub(in crate::content::host) fn on_cards_to_discard_l11_vm_c(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    cards: &[CardId],
    is_ordering_discard: bool,
) {
    hooks::on_cards_to_discard_l11_vm_c(ctx, player, cards, is_ordering_discard);
}

pub(in crate::content::host) fn on_hand_card_removed_l11_vm_c(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: CardId,
) {
    hooks::on_hand_card_removed_l11_vm_c(ctx, player, card);
}

pub(in crate::content::host) fn on_outer_settled_l11_vm_c(ctx: &mut Ctx<'_>) {
    hooks::on_outer_settled_l11_vm_c(ctx);
}

pub(in crate::content::host) fn sha_begin_step_l11_vm_c(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
    priority_ge_6: bool,
) -> bool {
    hooks::sha_begin_step_l11_vm_c(ctx, sha_frame_id, source, target, sha_card, priority_ge_6)
}
