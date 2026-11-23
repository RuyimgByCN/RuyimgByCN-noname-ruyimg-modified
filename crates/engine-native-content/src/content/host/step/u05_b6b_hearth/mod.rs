//! Batch U05-6b (`shahai-hearth` Part 2): 107 清单技能 + 4 子技（`xieneng` .. `hongxi` + `stone_chenshui`）。

use super::u05_hearth::{
    base_card_name, emit_hearth_virtual, exact_base_name, flag, gain_card_to_hand,
    has_any_skill, has_skill_hearth, in_attack_range_hearth, is_delayed_trick_hearth, set_flag,
    spawn_card_by_base,
};
use super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "asks.rs"]
pub(super) mod asks;
#[path = "group_a.rs"]
pub(super) mod group_a;
#[path = "group_b.rs"]
pub(super) mod group_b;
#[path = "group_c.rs"]
pub(super) mod group_c;
#[path = "group_d.rs"]
pub(super) mod group_d;
#[path = "group_e.rs"]
pub(super) mod group_e;
#[path = "group_f.rs"]
pub(super) mod group_f;
#[path = "hooks.rs"]
pub(super) mod hooks;

pub(in crate::content::host) use asks::ask_u05_b6b;
pub(crate) use hooks::{
    b6b_dying_save_skills, b6b_on_card_use_after, b6b_on_draw_begin, b6b_on_game_start,
    b6b_on_hand_card_removed, b6b_on_outer_settled, b6b_on_phase_end, b6b_on_phase_use_end,
    b6b_on_player_died, b6b_on_skill_removed, b6b_on_turn_begin, b6b_resolve_dying_save,
    hearth_response_skill_jingxiang, host_on_judge_pre_hearth, host_on_judge_rewrite_pre_hearth,
    jingxiang_spawn_response,
};

pub(super) fn b6b_flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    flag(core, p, key)
}

pub(super) fn set_b6b_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    set_flag(core, p, key, val);
}

pub(super) fn clear_b6b_flag(core: &mut GameStateCore, p: PlayerId, key: &str) {
    set_flag(core, p, key, 0);
}

pub(super) fn inc_b6b_flag(core: &mut GameStateCore, p: PlayerId, key: &str) -> u32 {
    let next = b6b_flag(core, p, key).saturating_add(1);
    set_b6b_flag(core, p, key, next);
    next
}

pub(super) fn hs_b6b_cnt(core: &GameStateCore, p: PlayerId, sk: &str) -> u32 {
    b6b_flag(core, p, &format!("hs_b6b_cnt:{sk}"))
}

pub(super) fn hs_b6b_inc(core: &mut GameStateCore, p: PlayerId, sk: &str) -> u32 {
    inc_b6b_flag(core, p, &format!("hs_b6b_cnt:{sk}"))
}

pub(super) fn hs_b6b_dec(core: &mut GameStateCore, p: PlayerId, sk: &str) {
    let cur = hs_b6b_cnt(core, p, sk);
    if cur > 0 {
        set_b6b_flag(core, p, &format!("hs_b6b_cnt:{sk}"), cur - 1);
    }
}

pub(super) fn has_skill_b6b(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog.skill_by_name(name).is_some_and(|d| {
        matches!(
            catalog.skills[d as usize].behavior.as_host_behavior(),
            Some(
                HostBehavior::U05Hearth2BatchA
                    | HostBehavior::U05Hearth2BatchB
                    | HostBehavior::U05Hearth2BatchC
                    | HostBehavior::U05Hearth2BatchD
                    | HostBehavior::U05Hearth2BatchE
                    | HostBehavior::U05Hearth2BatchF
            )
        ) && core.has_skill(p, d)
    })
}

pub(super) fn is_b6b_red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

pub(super) fn b6b_resolve_src_card(
    core: &GameStateCore,
    catalog: &Catalog,
    u: &CardUseData,
    last_vsrc: u32,
) -> Option<CardId> {
    let user = u.player;
    let eff = u.effective_kind(catalog);
    if let Some(cid) = u.card {
        let cname = exact_base_name(catalog.cards[cid as usize].name);
        let was_shadow = flag(core, user, &format!("hs_wxuying_was_shadow:{cid}")) > 0
            || flag(core, user, &format!("hs_wxuying_kind:{cid}")) > 0;
        if cname == "hsfashu_anyingjingxiang" && was_shadow {
            let src_plus = flag(core, user, &format!("hs_wxuying_src_card:{cid}"));
            return (src_plus > 0)
                .then(|| src_plus - 1)
                .filter(|&sc| u.use_as.is_none() || u.use_as == Some(catalog.card_kind(sc)));
        }
        return if u.use_as.is_none() || u.use_as == Some(catalog.card_kind(cid)) {
            Some(cid)
        } else {
            None
        };
    }
    let vsrc = if last_vsrc > 0 {
        last_vsrc
    } else {
        flag(core, user, "hs_virtual_card_src")
    };
    if vsrc > 0
        && catalog
            .cards
            .get((vsrc - 1) as usize)
            .is_some_and(|c| c.kind == eff)
    {
        Some(vsrc - 1)
    } else {
        None
    }
}

pub(super) fn b6b_card_name_key(
    core: &GameStateCore,
    catalog: &Catalog,
    u: &CardUseData,
    last_vsrc: u32,
) -> String {
    let eff = u.effective_kind(catalog);
    if eff.is_sha_like() {
        return "sha".to_string();
    }
    if let Some(cid) = b6b_resolve_src_card(core, catalog, u, last_vsrc) {
        let def = &catalog.cards[cid as usize];
        if let Some(orig) = def.original_source {
            return orig.raw_symbol.to_string();
        }
        let base = base_card_name(def.name);
        return match base {
            "无中生有" => "wuzhong".to_string(),
            "桃" => "tao".to_string(),
            "酒" => "jiu".to_string(),
            "过河拆桥" => "guohe".to_string(),
            "顺手牵羊" => "shunshou".to_string(),
            "决斗" => "juedou".to_string(),
            other => other.to_string(),
        };
    }
    match eff {
        CardKind::Tao => "tao".to_string(),
        CardKind::Jiu => "jiu".to_string(),
        CardKind::Wuzhong => "wuzhong".to_string(),
        CardKind::Duel => "juedou".to_string(),
        CardKind::Dismantle => "guohe".to_string(),
        CardKind::Snatch => "shunshou".to_string(),
        CardKind::Nullify => "wuxie".to_string(),
        other => format!("kind:{}", crate::archive::kind_code(other)),
    }
}

pub(super) fn is_b6b_card_target_legal(
    state: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    tgt: PlayerId,
    name_cid: Option<CardId>,
    cancel_cid: Option<CardId>,
    eff_kind: CardKind,
    nature: DamageNature,
) -> bool {
    if !state.player(tgt).alive {
        return false;
    }
    if let Some(c) = cancel_cid {
        if crate::rvm_bridge::modifiers::eval_card_target_canceled(
            state, catalog, user, tgt, c, eff_kind, nature,
        ) {
            return false;
        }
    }
    let pl = state.player(tgt);
    if let Some(c) = name_cid {
        let raw = catalog.cards[c as usize].name;
        let exact = exact_base_name(raw);
        let base = base_card_name(raw);
        if base == "shihuawuqi" {
            return user == tgt;
        }
        if base == "huogong" {
            return !pl.hand.is_empty();
        }
        if base == "shandianjian" {
            return user != tgt && !pl.hand.is_empty();
        }
        if exact == "hsdusu_xueji" {
            return user != tgt && !pl.equips.is_empty();
        }
        if exact == "hsdusu_kuyecao" {
            return user != tgt && !pl.yinshen_active;
        }
        if exact == "hsdusu_shinancao" {
            return user != tgt && !has_skill_hearth(state, catalog, tgt, "hsdusu_shinancao");
        }
        if exact == "hsdusu_huoyanhua" {
            return user != tgt && in_attack_range_hearth(state, catalog, user, tgt);
        }
        if matches!(exact, "hsqingyu_feibiao" | "hszuzhou_nvwudeganguo" | "hszuzhou_guhuo") {
            return user != tgt && (!pl.hand.is_empty() || !pl.equips.is_empty());
        }
        if exact == "hsmengjing_huanxiaojiemei" {
            return pl.hp < pl.max_hp as i32;
        }
        if exact == "hsmengjing_mengjing" {
            let has_le = pl.judge.iter().any(|&jc| catalog.card_kind(jc) == CardKind::Le);
            return user != tgt && (!has_le || !pl.equips.is_empty());
        }
        if exact == "hszuzhou_nvwudexuetu" {
            return user != tgt && !has_any_skill(state, catalog, tgt, "zhoujiang");
        }
    }
    if eff_kind.is_sha_like() {
        return user != tgt;
    }
    if eff_kind.equip_slot().is_some() {
        return user == tgt;
    }
    match eff_kind {
        CardKind::Tao => user == tgt && pl.hp < pl.max_hp as i32,
        CardKind::Jiu | CardKind::Tianxianjiu => user == tgt,
        CardKind::Zhiliaobo => pl.hp < pl.max_hp as i32,
        CardKind::Shijieshu | CardKind::Chain | CardKind::Linghunzhihuo => true,
        CardKind::Wuzhong
        | CardKind::Chuansongmen
        | CardKind::Dunpaigedang
        | CardKind::Tanshezhiren
        | CardKind::Xingjiegoutong
        | CardKind::Jihuocard
        | CardKind::Shenmiguo
        | CardKind::Hufu => user == tgt,
        CardKind::Duel | CardKind::Liuxinghuoyu => user != tgt,
        CardKind::Dismantle | CardKind::Zhaomingdan | CardKind::Snatch => {
            user != tgt && (!pl.hand.is_empty() || !pl.equips.is_empty() || !pl.judge.is_empty())
        }
        CardKind::Le | CardKind::Bing => {
            user != tgt && !pl.judge.iter().any(|&jc| catalog.card_kind(jc) == eff_kind)
        }
        CardKind::ShanDian => {
            user == tgt && !pl.judge.iter().any(|&jc| catalog.card_kind(jc) == CardKind::ShanDian)
        }
        _ => !is_delayed_trick_hearth(eff_kind) && user != tgt,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_hearth_machine(
    ctx: &mut Ctx<'_>,
    behavior: HostBehavior,
    def_key: &str,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
) {
    let m = Box::new(HostMachine {
        def_key: def_key.to_string(),
        behavior,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    });
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger {
            frame: ctx.res.stack.last().map_or(crate::resolution::NO_FRAME, |f| f.id),
        },
        machine: m,
        awaiting_answer: false,
    });
}

pub(super) fn stash_card(core: &mut GameStateCore, owner: PlayerId, pile_name: &str, cid: CardId) {
    core.vm_aux
        .piles
        .entry((owner, pile_name.to_string()))
        .or_default()
        .push(cid);
}

pub(super) fn pop_stashed_card(
    core: &mut GameStateCore,
    owner: PlayerId,
    pile_name: &str,
) -> Option<CardId> {
    let remove_key = core
        .vm_aux
        .piles
        .get(&(owner, pile_name.to_string()))
        .map_or(true, |v| v.is_empty());
    if remove_key {
        core.vm_aux.piles.remove(&(owner, pile_name.to_string()));
        return None;
    }
    let vec = core
        .vm_aux
        .piles
        .get_mut(&(owner, pile_name.to_string()))?;
    let c = vec.pop();
    if vec.is_empty() {
        core.vm_aux.piles.remove(&(owner, pile_name.to_string()));
    }
    c
}

pub(super) fn sha_target_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && !core.player(p).face_down
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core,
                    catalog,
                    owner,
                    p,
                    0,
                    CardKind::Sha,
                    DamageNature::Normal,
                )
        })
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn emit_free_sha_no_count(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId) {
    set_b6b_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
    emit_hearth_virtual(ctx, owner, CardKind::Sha, vec![target]);
}

pub(super) fn can_jingxiang_card_respond(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
    resp_ctx: u8,
) -> bool {
    let kind = super::u05_hearth_a_tail::effective_hearth_card(core, catalog, owner, cid).1;
    match resp_ctx {
        1 => kind == CardKind::Shan,
        2 => kind.is_sha_like(),
        3 => kind == CardKind::Nullify,
        4 => kind == CardKind::Tao,
        5 => matches!(kind, CardKind::Tao | CardKind::Jiu),
        _ => false,
    }
}

pub(super) fn tuteng_pool(awake: bool) -> Vec<u8> {
    if awake {
        (1..=8).collect()
    } else {
        (1..=4).collect()
    }
}

pub(super) fn tuteng_owned(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<u8> {
    (1..=8u8)
        .filter(|&i| has_any_skill(core, catalog, p, &format!("tuteng{i}")))
        .collect()
}

pub(super) fn spawn_random_dream_card(ctx: &mut Ctx<'_>, recipient: PlayerId) {
    const LIST: [&str; 5] = [
        "hsmengjing_feicuiyoulong",
        "hsmengjing_huanxiaojiemei",
        "hsmengjing_suxing",
        "hsmengjing_mengye",
        "hsmengjing_mengjing",
    ];
    let avail: Vec<&str> = LIST
        .iter()
        .copied()
        .filter(|&name| {
            ctx.core
                .detached_cards
                .iter()
                .any(|&(c, _)| super::u05_hearth::exact_base_name(ctx.catalog.cards[c as usize].name) == name)
        })
        .collect();
    if let Some(pick) = super::u05_hearth::rng_pick(ctx, &avail) {
        if let Some(c) = spawn_card_by_base(ctx, pick) {
            gain_card_to_hand(ctx, recipient, c);
        }
    }
}

pub(super) fn trigger_meta(name: &str, behavior: HostBehavior) -> Option<TriggerMeta> {
    match behavior {
        HostBehavior::U05Hearth2BatchA => group_a::trigger_meta_a(name),
        HostBehavior::U05Hearth2BatchB => group_b::trigger_meta_b(name),
        HostBehavior::U05Hearth2BatchC => group_c::trigger_meta_c(name),
        HostBehavior::U05Hearth2BatchD => group_d::trigger_meta_d(name),
        HostBehavior::U05Hearth2BatchE => group_e::trigger_meta_e(name),
        HostBehavior::U05Hearth2BatchF => group_f::trigger_meta_f(name),
        _ => None,
    }
}

pub(super) fn limits(name: &str, behavior: HostBehavior) -> SkillLimits {
    match behavior {
        HostBehavior::U05Hearth2BatchA => group_a::limits_a(name),
        HostBehavior::U05Hearth2BatchB => group_b::limits_b(name),
        HostBehavior::U05Hearth2BatchC => group_c::limits_c(name),
        HostBehavior::U05Hearth2BatchD => group_d::limits_d(name),
        HostBehavior::U05Hearth2BatchE => group_e::limits_e(name),
        HostBehavior::U05Hearth2BatchF => group_f::limits_f(name),
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_u05_b6b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05Hearth2BatchA => group_a::filter_a(skill, ctx, owner, event),
        HostBehavior::U05Hearth2BatchB => group_b::filter_b(skill, ctx, owner, event),
        HostBehavior::U05Hearth2BatchC => group_c::filter_c(skill, ctx, owner, event),
        HostBehavior::U05Hearth2BatchD => group_d::filter_d(skill, ctx, owner, event),
        HostBehavior::U05Hearth2BatchE => group_e::filter_e(skill, ctx, owner, event),
        HostBehavior::U05Hearth2BatchF => group_f::filter_f(skill, ctx, owner, event),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u05_b6b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive
        || ctx.state.turn.current != owner
        || ctx.state.turn.phase != crate::event::Phase::Play
        || ctx.catalog.is_tieji_sealed(ctx.state, owner)
    {
        return None;
    }
    let ok = match skill.behavior {
        HostBehavior::U05Hearth2BatchA => group_a::activate_a(skill, ctx, owner),
        HostBehavior::U05Hearth2BatchB => group_b::activate_b(skill, ctx, owner),
        HostBehavior::U05Hearth2BatchC => group_c::activate_c(skill, ctx, owner),
        HostBehavior::U05Hearth2BatchD => group_d::activate_d(skill, ctx, owner),
        HostBehavior::U05Hearth2BatchE => group_e::activate_e(skill, ctx, owner),
        HostBehavior::U05Hearth2BatchF => group_f::activate_f(skill, ctx, owner),
        _ => None,
    };
    if ok == Some(true) {
        Some(Box::new(HostMachine {
            def_key: skill.name.clone(),
            behavior: skill.behavior,
            owner,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }))
    } else {
        None
    }
}

pub(in crate::content::host) fn has_phase_use_activation_b6b(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    match behavior {
        HostBehavior::U05Hearth2BatchA => group_a::has_active_a(name),
        HostBehavior::U05Hearth2BatchB => group_b::has_active_b(name),
        HostBehavior::U05Hearth2BatchC => group_c::has_active_c(name),
        HostBehavior::U05Hearth2BatchD => group_d::has_active_d(name),
        HostBehavior::U05Hearth2BatchE => group_e::has_active_e(name),
        HostBehavior::U05Hearth2BatchF => group_f::has_active_f(name),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_u05_b6b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut targets = Vec::new();
    let mut cards = Vec::new();
    match event {
        EventData::Damage(d) => {
            target = Some(d.target);
            card = d.card;
            if let Some(src) = d.source {
                targets.push(src);
            }
        }
        EventData::CardUsed(u) => {
            target = Some(u.player);
            card = u.card;
            targets = u.targets.clone();
        }
        EventData::Recover { player, .. } => {
            target = Some(*player);
        }
        EventData::Death { player, killer } => {
            target = Some(*player);
            card = killer.map(u32::from);
        }
        EventData::TurnBegin { player }
        | EventData::TurnEndWindow { player, .. }
        | EventData::Phase { player, .. }
        | EventData::TurnOvered { player, .. } => {
            target = Some(*player);
        }
        EventData::CardsDrawn { player, cards: cs }
        | EventData::CardsDiscarded { player, cards: cs }
        | EventData::CardsLost { player, cards: cs, .. } => {
            target = Some(*player);
            cards = cs.clone();
        }
        EventData::CardsMoved { cards: cs, from, to } => {
            target = Some(*to);
            targets.push(*from);
            cards = cs.clone();
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_u05_b6b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::U05Hearth2BatchA => group_a::step_a(m, ctx),
        HostBehavior::U05Hearth2BatchB => group_b::step_b(m, ctx),
        HostBehavior::U05Hearth2BatchC => group_c::step_c(m, ctx),
        HostBehavior::U05Hearth2BatchD => group_d::step_d(m, ctx),
        HostBehavior::U05Hearth2BatchE => group_e::step_e(m, ctx),
        HostBehavior::U05Hearth2BatchF => group_f::step_f(m, ctx),
        _ => Flow::Done,
    }
}
