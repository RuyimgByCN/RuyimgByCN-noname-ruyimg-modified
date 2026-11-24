//! `L11` (`太虚幻境`) cross-package NPC skills Part B (`satomi_luodao`..=`zongkui`,
//! excluding `xinfu_sidao` preserved under `HostBehavior::L11GeneralBossC`), plus companion
//! derived skills (`twqingce`, `twsaotao`, `twmouli`, `twzhian`, `twxiongjun`, `baonvezhi`,
//! `paiyi`, `juedi`, `xinfu_zengdao2`, `meiyingqiang`, `taipingyaoshu`).

#[path = "active_a.rs"]
mod active_a;
#[path = "active_b.rs"]
mod active_b;
#[path = "asks_a.rs"]
mod asks_a;
#[path = "asks_b.rs"]
mod asks_b;
#[path = "combat_damage.rs"]
mod combat_damage;
#[path = "combat_step_b.rs"]
mod combat_step_b;
#[path = "helpers.rs"]
pub(super) mod helpers;
#[path = "meta.rs"]
pub(super) mod meta;
pub(in crate::content::host) use meta::{limits_l11_npc_b, trigger_meta_l11_npc_b};
#[path = "phase_hooks.rs"]
mod phase_hooks;
#[path = "phase_step_b.rs"]
mod phase_step_b;
#[path = "phase_turn_death.rs"]
mod phase_turn_death;
#[path = "reactive_misc.rs"]
mod reactive_misc;
#[path = "reactive_step_b.rs"]
mod reactive_step_b;

use super::super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceOption, ChoiceSpec};
use crate::event::{DamageData, EventData};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_SATOMI_DISCARD_SHAN: u32 = 1;
pub(super) const ASK_SATOMI_DISCARD_SELF: u32 = 2;
pub(super) const ASK_SBLIEGONG_CONFIRM: u32 = 3;
pub(super) const ASK_SHANGYI_TARGET: u32 = 10;
pub(super) const ASK_SHANGYI_DISCARD_BLACK: u32 = 11;
pub(super) const ASK_SHEFU_NAME: u32 = 12;
pub(super) const ASK_SHEFU_CARD: u32 = 13;
pub(super) const ASK_SHEFU2_CONFIRM: u32 = 14;
pub(super) const ASK_SHENSU1_TARGET: u32 = 15;
pub(super) const ASK_SHENSU2_CARD: u32 = 16;
pub(super) const ASK_SHENSU2_TARGET: u32 = 17;
pub(super) const ASK_SHUNSHI_CARD: u32 = 18;
pub(super) const ASK_SHUNSHI_TARGET: u32 = 19;
pub(super) const ASK_SPSHIJI_CONFIRM: u32 = 20;
pub(super) const ASK_SPWENJI_TARGET: u32 = 21;
pub(super) const ASK_SPWENJI_GIVE: u32 = 22;
pub(super) const ASK_SPYISHI_EQUIP: u32 = 23;
pub(super) const ASK_TIANMING_DISCARD: u32 = 24;
pub(super) const ASK_TIANMING_OTHER_CONFIRM: u32 = 25;
pub(super) const ASK_TIANMING_OTHER_DISCARD: u32 = 26;
pub(super) const ASK_TIANSHU_CARD: u32 = 27;
pub(super) const ASK_TIANSHU_TARGET: u32 = 28;
pub(super) const ASK_TUIFENG_CARDS: u32 = 29;
pub(super) const ASK_ZHANYI_DISCARD: u32 = 30;
pub(super) const ASK_ZHANYI_BASIC_KIND: u32 = 31;
pub(super) const ASK_ZHANYI_BASIC_CARD: u32 = 32;
pub(super) const ASK_ZHANYI_BASIC_TARGET: u32 = 33;
pub(super) const ASK_ZHANYI_EQUIP_DISCARD: u32 = 34;
pub(super) const ASK_ZHENDU_DISCARD: u32 = 35;
pub(super) const ASK_ZHIDAO_HAND: u32 = 36;
pub(super) const ASK_ZHIDAO_EQUIP: u32 = 37;
pub(super) const ASK_ZHIDAO_JUDGE: u32 = 38;
pub(super) const ASK_ZHIMAN_CARD: u32 = 39;
pub(super) const ASK_ZHIYAN_TARGET: u32 = 40;
pub(super) const ASK_ZHONGYONG_TARGET: u32 = 41;
pub(super) const ASK_ZHONGYONG_USE_SHA: u32 = 42;
pub(super) const ASK_ZHUIDE_TARGET: u32 = 43;
pub(super) const ASK_ZHUIDE_BASICS: u32 = 44;
pub(super) const ASK_ZILI_MODE: u32 = 45;
pub(super) const ASK_PAIYI_TARGET: u32 = 46;
pub(super) const ASK_PAIYI_PILE: u32 = 47;
pub(super) const ASK_ZONGKUI_TARGET: u32 = 48;
pub(super) const ASK_SHIZHI_CARD: u32 = 49;
pub(super) const ASK_SHIZHI_TARGET: u32 = 50;
pub(super) const ASK_TWWUSHEN_CARD: u32 = 51;
pub(super) const ASK_TWWUSHEN_TARGET: u32 = 52;
pub(super) const ASK_MEIYINGQIANG_CARD: u32 = 53;
pub(super) const ASK_MEIYINGQIANG_TARGET: u32 = 54;

pub(super) const ASK_TWFENMING_TARGET: u32 = 100;
pub(super) const ASK_TWFENMING_MODE: u32 = 101;
pub(super) const ASK_TWFENMING_DISCARD: u32 = 102;
pub(super) const ASK_TWHENGJIANG_CONFIRM: u32 = 103;
pub(super) const ASK_TWHONGJU_SWAP_HAND: u32 = 104;
pub(super) const ASK_TWHONGJU_SWAP_PILE: u32 = 105;
pub(super) const ASK_TWHONGJU_SAOTAO: u32 = 106;
pub(super) const ASK_TWQINGCE_PILE: u32 = 107;
pub(super) const ASK_TWQINGCE_TARGET: u32 = 108;
pub(super) const ASK_TWQINGCE_DISCARD: u32 = 109;
pub(super) const ASK_TWJIEFAN_TARGET: u32 = 110;
pub(super) const ASK_TWJIEFAN_WEAPON: u32 = 111;
pub(super) const ASK_TWJUNTUN_TARGET: u32 = 112;
pub(super) const ASK_TWLIANCAI_TARGET: u32 = 113;
pub(super) const ASK_TWLIANCAI_EQUIP: u32 = 114;
pub(super) const ASK_TWLIANCAI_DRAW: u32 = 115;
pub(super) const ASK_TWLINGFA_MODE: u32 = 116;
pub(super) const ASK_TWLINGFA_SHA_DISCARD: u32 = 117;
pub(super) const ASK_TWLINGFA_TAO_GIVE: u32 = 118;
pub(super) const ASK_TWZHIAN_MODE: u32 = 119;
pub(super) const ASK_TWLINGLU_TARGET: u32 = 120;
pub(super) const ASK_TWMOU_CARD: u32 = 121;
pub(super) const ASK_TWMOU_TARGET: u32 = 122;
pub(super) const ASK_TWQIANXI_CONFIRM: u32 = 123;
pub(super) const ASK_TWQIANXI_DISCARD: u32 = 124;
pub(super) const ASK_TWQIANXI_TARGET: u32 = 125;
pub(super) const ASK_TWQIRANG_TARGET: u32 = 126;
pub(super) const ASK_TWWUHUN_TARGETS: u32 = 127;
pub(super) const ASK_TWXIONGXI_DISCARD: u32 = 128;
pub(super) const ASK_TWXIONGXI_TARGET: u32 = 129;
pub(super) const ASK_TWXIONGZHENG_ACTION: u32 = 130;
pub(super) const ASK_TWXIONGZHENG_SHA_TARGETS: u32 = 131;
pub(super) const ASK_TWXIONGZHENG_DRAW_TARGETS: u32 = 132;
pub(super) const ASK_TWXIONGZHENG_TARGET: u32 = 133;
pub(super) const ASK_TWXUANHUO_CARDS: u32 = 134;
pub(super) const ASK_TWXUANHUO_TARGET: u32 = 135;
pub(super) const ASK_TWXUANHUO_DUEL_TARGET: u32 = 136;
pub(super) const ASK_TWXUANHUO_TARGET_CHOICE: u32 = 137;
pub(super) const ASK_TWXUANHUO_TAKE_CARDS: u32 = 138;
pub(super) const ASK_TWXUECHANG_TARGET: u32 = 139;
pub(super) const ASK_TWXUECHANG_PINDIAN_OWNER: u32 = 140;
pub(super) const ASK_TWXUECHANG_PINDIAN_TARGET: u32 = 141;
pub(super) const ASK_TWXUECHANG_GAIN: u32 = 142;
pub(super) const ASK_TWXUHE_TARGET_CHOOSE: u32 = 143;
pub(super) const ASK_TWYILIE_MODE: u32 = 144;
pub(super) const ASK_TWYINGJIA_CARD: u32 = 145;
pub(super) const ASK_TWYINGJIA_TARGET: u32 = 146;
pub(super) const ASK_TWYUANHU_EQUIP: u32 = 147;
pub(super) const ASK_TWYUANHU_TARGET: u32 = 148;
pub(super) const ASK_TWYUANHU_WEAPON_TGT: u32 = 149;
pub(super) const ASK_TWYUANHU_WEAPON_DISCARD: u32 = 150;
pub(super) const ASK_TWYUHUA_GUANXING: u32 = 151;
pub(super) const ASK_TWYUHUA_DRAW: u32 = 152;
pub(super) const ASK_TWZHENGRONG_TARGET: u32 = 153;
pub(super) const ASK_TWZHENGRONG_CARD: u32 = 154;
pub(super) const ASK_WAISHI_CARDS: u32 = 155;
pub(super) const ASK_WAISHI_TARGET: u32 = 156;
pub(super) const ASK_WAISHI_TARGET_CARDS: u32 = 157;
pub(super) const ASK_WEIFENG_TARGET: u32 = 158;
pub(super) const ASK_WEIFENG3_GAIN: u32 = 159;
pub(super) const ASK_WEIJING_MODE: u32 = 160;
pub(super) const ASK_WEIJING_TARGET: u32 = 161;
pub(super) const ASK_WULIE_TARGETS: u32 = 162;
pub(super) const ASK_XIANFU_TARGET: u32 = 163;
pub(super) const ASK_XINFENYUE_TARGET: u32 = 164;
pub(super) const ASK_XINFENYUE_PINDIAN_OWNER: u32 = 165;
pub(super) const ASK_XINFENYUE_PINDIAN_TARGET: u32 = 166;
pub(super) const ASK_XINFENYUE_GAIN: u32 = 167;
pub(super) const ASK_XINFU_DAIGONG_GIVE: u32 = 168;
pub(super) const ASK_XINFU_KANNAN_TARGET: u32 = 169;
pub(super) const ASK_XINFU_KANNAN_PINDIAN_OWNER: u32 = 170;
pub(super) const ASK_XINFU_KANNAN_PINDIAN_TARGET: u32 = 171;
pub(super) const ASK_XINFU_ZENGDAO_CARDS: u32 = 172;
pub(super) const ASK_XINFU_ZENGDAO_TARGET: u32 = 173;
pub(super) const ASK_XINFU_ZHAOXIN_CARDS: u32 = 174;
pub(super) const ASK_XINFU_ZHAOXIN_PILE: u32 = 175;
pub(super) const ASK_XINFU_ZHAOXIN_TARGET_GAIN: u32 = 176;
pub(super) const ASK_XINFU_ZHAOXIN_DAMAGE: u32 = 177;
pub(super) const ASK_XINJIEWEI_DISCARD: u32 = 178;
pub(super) const ASK_XINJIEWEI_SRC: u32 = 179;
pub(super) const ASK_XINJIEWEI_CARD: u32 = 180;
pub(super) const ASK_XINJIEWEI_DST: u32 = 181;
pub(super) const ASK_XINJUSHOU_CARD: u32 = 182;
pub(super) const ASK_XINKUANGFU_TARGET: u32 = 183;
pub(super) const ASK_XINKUANGFU_DISCARD_EQUIP: u32 = 184;
pub(super) const ASK_XINKUANGFU_SHA_TARGET: u32 = 186;
pub(super) const ASK_XINKUANGFU_DISCARD_TWO: u32 = 187;
pub(super) const ASK_XINSHENG_CONFIRM: u32 = 188;
pub(super) const ASK_YECHOU_TARGET: u32 = 189;
pub(super) const ASK_YINBING_CARDS: u32 = 190;
pub(super) const ASK_JUEDI_TARGET: u32 = 191;
pub(super) const ASK_YISE_RECOVER: u32 = 192;
pub(super) const ASK_YJJIANSHU_CARD: u32 = 193;
pub(super) const ASK_YJJIANSHU_TARGETS: u32 = 194;
pub(super) const ASK_YJJIANSHU_PINDIAN_1: u32 = 195;
pub(super) const ASK_YJJIANSHU_PINDIAN_2: u32 = 196;
pub(super) const ASK_YONGJIN_SRC: u32 = 197;
pub(super) const ASK_YONGJIN_CARD: u32 = 198;
pub(super) const ASK_YONGJIN_DST: u32 = 199;
pub(super) const ASK_YONGSI2_DISCARD: u32 = 200;
pub(super) const ASK_YUCE_SHOW: u32 = 201;
pub(super) const ASK_YUCE_SOURCE_DISCARD: u32 = 202;
pub(super) const ASK_TWZHIAN_DISCARD_HAND: u32 = 203;

pub(super) fn build_ask_l11_npc_b(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let ask_id = helpers::flag(core, owner, "l11b_ask_id");
    if ask_id < 100 {
        asks_a::build_ask_a(ask_id, stage, owner, target, targets, card, cards, core, catalog)
    } else {
        asks_b::build_ask_b(ask_id, stage, owner, target, targets, card, cards, core, catalog)
    }
}

pub(in crate::content::host) fn ask_l11_npc_b(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) =
        build_ask_l11_npc_b(stage, owner, target, targets, card, cards, core, catalog)?;
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

pub(in crate::content::host) fn has_phase_use_activation_l11_npc_b(name: &str) -> bool {
    active_a::has_phase_use_a(name) || active_b::has_phase_use_b(name)
}

pub(in crate::content::host) fn activate_l11_npc_b(
    skill: &HostSkillBehavior,
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<HostMachine> {
    active_a::activate_a(skill, state, catalog, owner)
        .or_else(|| active_b::activate_b(skill, state, catalog, owner))
}

pub(in crate::content::host) fn filter_l11_npc_b_with_frames(
    skill: &HostSkillBehavior,
    ctx: &crate::catalog::QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let ev = match (event, frame) {
        (EventData::Damage(_), Some(f)) if matches!(f.data, EventData::DamageWindow { .. }) => {
            &f.data
        }
        _ => event,
    };
    combat_damage::filter_combat_damage(skill, ctx, frames, frame, owner, ev)
        || phase_turn_death::filter_phase_turn_death(skill, ctx, frames, frame, owner, ev)
        || reactive_misc::filter_reactive_misc(skill, ctx, frames, frame, owner, ev)
}

pub(in crate::content::host) fn machine_l11_npc_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut m = HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11CrossNpcB,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    };
    match event {
        EventData::CardUsed(u) => {
            if skill.name == "zhanyi" {
                let tgt = u.targets.first().copied().unwrap_or(u.player);
                m.target = Some(tgt);
                m.card = u.card;
                m.targets = vec![u.player, tgt];
            } else {
                m.target = Some(u.player);
                m.card = u.card;
                m.targets = u.targets.clone();
            }
        }
        EventData::CardResponded { player, card, .. } => {
            m.target = Some(*player);
            m.card = Some(*card);
        }
        EventData::DamageWindow { root, window } => {
            m.cards = vec![*root, *window as CardId, 0];
        }
        EventData::Dying { player, .. } => {
            m.target = Some(*player);
        }
        EventData::Death { player, killer } => {
            m.target = Some(*player);
            m.targets = (*killer).into_iter().collect();
        }
        EventData::Phase { player, phase, .. } => {
            m.target = Some(*player);
            m.card = Some(*phase as CardId);
        }
        EventData::ShaDodged {
            card,
            source,
            target,
        } => {
            m.target = Some(*target);
            m.card = *card;
            m.targets = vec![*source, *target];
        }
        EventData::CardsLost { player, cards, .. } => {
            m.target = Some(*player);
            m.cards = cards.clone();
        }
        EventData::CardsMoved { cards, from, to } => {
            m.target = Some(*to);
            m.targets = vec![*from, *to];
            m.cards = cards.clone();
        }
        EventData::Equipped { player, card, .. } => {
            m.target = Some(*player);
            m.card = Some(*card);
        }
        EventData::TurnOvered { player, face_down } => {
            m.target = Some(*player);
            m.card = Some(u32::from(*face_down));
        }
        _ => {}
    }
    m
}

pub(super) fn handle_l11_npc_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let _ = helpers::re_exports_check(ctx, m.owner, &[]);
    match m.def_key.as_str() {
        "twxuechang" | "xinfu_kannan" if m.stage == HostStage::Act && m.cards.len() == 3 => {
            combat_damage::step_combat_damage(m, ctx)
        }
        "xinfu_zhaoxin" if m.target.is_some() => phase_turn_death::step_phase_turn_death(m, ctx),
        "shangyi"
        | "twqingce"
        | "twjiefan"
        | "twxiongxi"
        | "twxuechang"
        | "twyuanhu"
        | "waishi"
        | "weijing"
        | "twmouli"
        | "shizhi"
        | "twwushen" => active_a::step_active_a(m, ctx),
        "xinfenyue"
        | "xinfu_kannan"
        | "xinfu_zengdao"
        | "xinfu_zhaoxin"
        | "xinkuangfu"
        | "yjjianshu"
        | "yongjin"
        | "zhanyi"
        | "paiyi" => active_b::step_active_b(m, ctx),
        "satomi_luodao"
        | "sbliegong"
        | "spshiji"
        | "spxiangzhen"
        | "spyishi"
        | "tianming"
        | "twhengjiang"
        | "twhuangjin"
        | "twjianming"
        | "twqirang"
        | "twxiangyu"
        | "twxuhe"
        | "xinfu_daigong"
        | "xinfu_fuyin"
        | "xinfu_zengdao2"
        | "yise"
        | "zhidao"
        | "zhiman"
        | "zhongyong"
        | "taipingyaoshu" => combat_damage::step_combat_damage(m, ctx),
        "juedi"
        | "shefu"
        | "shensu"
        | "sptunjiang"
        | "spwenji"
        | "tianshu"
        | "twfenming"
        | "twhanyu"
        | "twhongju"
        | "twlingfa"
        | "twlingfa_sha"
        | "twlingfa_tao"
        | "twzhian"
        | "twlinglu"
        | "twqianxi"
        | "twqueshi"
        | "twxingqi"
        | "twxiongzheng"
        | "twxuanhuo"
        | "twyingjia"
        | "twzhengrong"
        | "wangzun"
        | "xianfu"
        | "xinjushou"
        | "yongsi"
        | "zhendu"
        | "zhiyan"
        | "zili"
        | "zongkui" => phase_turn_death::step_phase_turn_death(m, ctx),
        _ => reactive_misc::step_reactive_misc(m, ctx),
    }
}

pub(in crate::content::host) fn on_round_start_l11_npc_b(ctx: &mut Ctx<'_>) {
    phase_hooks::on_round_start_l11_npc_b(ctx);
}

pub(in crate::content::host) fn on_turn_begin_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    phase_hooks::on_turn_begin_l11_npc_b(ctx, player);
}

pub(in crate::content::host) fn on_draw_begin_step_l11_npc_b(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    pending: &mut i32,
) {
    phase_hooks::on_draw_begin_step_l11_npc_b(ctx, player, pending);
}

pub(in crate::content::host) fn on_card_use_after_l11_npc_b(
    ctx: &mut Ctx<'_>,
    fid: EventId,
    u: &crate::event::CardUseData,
) {
    phase_hooks::on_card_use_after_l11_npc_b(ctx, fid, u);
}

pub(in crate::content::host) fn on_phase_skipped_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    phase_hooks::on_phase_skipped_l11_npc_b(ctx, player);
}

pub(in crate::content::host) fn on_phase_use_end_l11_npc_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    phase_hooks::on_phase_use_end_l11_npc_b(ctx, player);
}

pub(in crate::content::host) fn on_phase_end_l11_npc_b(ctx: &mut Ctx<'_>) {
    phase_hooks::on_phase_end_l11_npc_b(ctx);
}

pub(in crate::content::host) fn on_turn_phase_after_l11_npc_b(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
) {
    phase_hooks::on_turn_phase_after_l11_npc_b(ctx, player);
}

pub(in crate::content::host) fn on_skill_removed_l11_npc_b(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) {
    phase_hooks::on_skill_removed_l11_npc_b(core, catalog, owner, skill_name);
}

pub(in crate::content::host) fn on_player_died_l11_npc_b(
    ctx: &mut Ctx<'_>,
    dead: PlayerId,
    killer: Option<PlayerId>,
) {
    phase_hooks::on_player_died_l11_npc_b(ctx, dead, killer);
}

pub(in crate::content::host) fn on_damage_end_l11_npc_b(ctx: &mut Ctx<'_>, d: &DamageData) {
    phase_hooks::on_damage_end_l11_npc_b(ctx, d);
}

pub(in crate::content::host) fn on_hp_changed_l11_npc_b(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    delta: i32,
) {
    phase_hooks::on_hp_changed_l11_npc_b(ctx, player, delta);
}

pub(in crate::content::host) fn on_equip_revoked_l11_npc_b(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) {
    phase_hooks::on_equip_revoked_l11_npc_b(core, catalog, player, card);
}

pub(in crate::content::host) fn on_outer_settled_l11_npc_b(ctx: &mut Ctx<'_>) {
    phase_hooks::on_outer_settled_l11_npc_b(ctx);
}

pub(in crate::content::host) fn on_host_event_l11_npc_b(ctx: &mut Ctx<'_>, event: &EventData) {
    phase_hooks::on_host_event_l11_npc_b(ctx, event);
}

pub(in crate::content::host) fn respond_skill_options_l11_npc_b(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    active_a::respond_skill_options_l11_npc_b(core, catalog, player, need, is_use, out);
}

pub(in crate::content::host) fn try_activate_respond_skill_l11_npc_b(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    fid: EventId,
    is_use: bool,
) -> Option<bool> {
    active_a::try_activate_respond_skill_l11_npc_b(ctx, player, def, need, fid, is_use)
}
