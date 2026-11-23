//! Batch U05-6a (`shahai-hearth` Part 1): 炉石传说卡牌与 109 项宿主技中枢模块。

use super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{DamageData, DamageNature, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

#[path = "u05_hearth_a.rs"]
pub(super) mod u05_hearth_a;
#[path = "u05_hearth_a_tail.rs"]
pub(super) mod u05_hearth_a_tail;
#[path = "u05_hearth_asks.rs"]
pub(super) mod u05_hearth_asks;
#[path = "u05_hearth_b.rs"]
pub(super) mod u05_hearth_b;
#[path = "u05_hearth_c.rs"]
pub(super) mod u05_hearth_c;
#[path = "u05_hearth_cards.rs"]
pub(super) mod u05_hearth_cards;
#[path = "u05_hearth_derived_cards.rs"]
pub(super) mod u05_hearth_derived_cards;
#[path = "u05_hearth_d.rs"]
pub(super) mod u05_hearth_d;
#[path = "u05_hearth_e.rs"]
pub(super) mod u05_hearth_e;
#[path = "u05_hearth_hooks.rs"]
pub(super) mod u05_hearth_hooks;

pub(in crate::content::host) use u05_hearth_asks::*;
pub(crate) use u05_hearth_cards::{hearth_try_named_ext_card, is_hearth_named_wuzhong_unwrapped};
pub(crate) use u05_hearth_hooks::{
    hearth_dying_save_skills, host_card_use_after_hearth, host_draw_begin_step_hearth,
    host_on_draw_machine_begin_hearth, host_on_equip_gained_hearth, host_on_equip_revoked_hearth,
    host_on_game_start_hearth, host_on_hand_card_removed_hearth, host_on_outer_settled_hearth,
    host_on_phase_end_hearth, host_on_phase_use_end_hearth, host_on_player_died_hearth,
    host_on_recover_before_hearth, host_on_round_start_hearth, host_sha_begin_step_hearth,
    host_sha_end_step_hearth, host_turn_before_skip_hearth, is_hearth_vanish_card_hook,
    on_skill_removed_hearth, resolve_hearth_dying_save,
};

/// 判定当前目录是否启用了 U05-6a `shahai-hearth` 形式包（区分仅载 `assets/cards.json` 的 `bridge_pack`）。
pub fn hearth_formal_enabled(catalog: &Catalog) -> bool {
    catalog.skill_by_name("shihuawuqi").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior()
            == Some(HostBehavior::U05HearthCards)
    }) || catalog.skill_by_name("hshuanyu").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior()
            == Some(HostBehavior::U05HearthBatchA)
    })
}

pub(super) fn base_card_name(raw: &str) -> &str {
    let raw = raw
        .strip_prefix("ht_")
        .or_else(|| raw.strip_prefix("hs_"))
        .unwrap_or(raw);
    if let Some((stem, tail)) = raw.rsplit_once('_') {
        if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) {
            return stem;
        }
    }
    raw
}

pub(super) fn exact_base_name(raw: &str) -> &str {
    if let Some((stem, tail)) = raw.rsplit_once('_') {
        if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) {
            return stem;
        }
    }
    raw
}

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn has_skill_hearth(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog.skill_by_name(name).is_some_and(|d| {
        matches!(
            catalog.skills[d as usize].behavior.as_host_behavior(),
            Some(
                HostBehavior::U05HearthCards
                    | HostBehavior::U05HearthBatchA
                    | HostBehavior::U05HearthBatchB
                    | HostBehavior::U05HearthBatchC
                    | HostBehavior::U05HearthBatchD
                    | HostBehavior::U05HearthBatchE
            )
        ) && core.has_skill(p, d)
    })
}

pub(super) fn has_any_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|d| core.has_skill(p, d))
}

pub(super) fn grant_skill(ctx: &mut Ctx<'_>, target: PlayerId, name: &str, temp: bool) {
    let Some(def) = ctx.catalog.skill_by_name(name) else {
        return;
    };
    ctx.insert_skill(
        (target, def),
        SkillInstance {
            owner: target,
            enabled: true,
            once_used: false,
            expires_end_of_turn: temp,
            rounds: 0,
        },
    );
}

pub(super) fn remove_skill(ctx: &mut Ctx<'_>, target: PlayerId, name: &str) {
    let Some(def) = ctx.catalog.skill_by_name(name) else {
        return;
    };
    if ctx.core.skills.remove(&(target, def)).is_some() {
        ctx.log(LogEvent::SkillToggled {
            player: target,
            skill: ctx.catalog.skill_name(def),
            enabled: false,
        });
        let _ = u05_hearth_hooks::on_skill_removed_hearth(ctx.core, ctx.catalog, target, name);
    }
}

pub(super) fn alive_others(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive)
        .collect()
}

pub(super) fn is_hearth_enemy(
    core: &GameStateCore,
    catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
) -> bool {
    a != b && core.player(b).alive && !core.is_friend_of(catalog, a, b)
}

pub(super) fn is_hearth_friend(
    core: &GameStateCore,
    catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
) -> bool {
    if !core.player(b).alive {
        return false;
    }
    if a == b {
        return true;
    }
    match &core.mode {
        crate::state::GameMode::Identity { roles } => {
            let ra = roles.get(a as usize).copied();
            let rb = roles.get(b as usize).copied();
            matches!(
                (ra, rb),
                (
                    Some(crate::state::Role::Lord | crate::state::Role::Loyalist),
                    Some(crate::state::Role::Lord | crate::state::Role::Loyalist)
                ) | (Some(crate::state::Role::Rebel), Some(crate::state::Role::Rebel))
            )
        }
        crate::state::GameMode::Versus { teams, .. } => {
            teams.get(a as usize) == teams.get(b as usize) && teams.get(a as usize).is_some()
        }
        crate::state::GameMode::Kingdoms => core.is_friend_of(catalog, a, b),
        _ => false,
    }
}

pub(super) fn is_max_hp(core: &GameStateCore, target: PlayerId) -> bool {
    let my_hp = core.player(target).hp;
    core.alive_players()
        .all(|p| p == target || core.player(p).hp < my_hp)
}

pub(super) fn is_max_handcard(core: &GameStateCore, target: PlayerId) -> bool {
    let max_h = core
        .alive_players()
        .map(|p| core.player(p).hand.len())
        .max()
        .unwrap_or(0);
    core.player(target).hand.len() >= max_h
}

pub(super) fn hearth_card_type_tag(catalog: &Catalog, cid: CardId) -> u8 {
    let kind = catalog.card_kind(cid);
    if kind.is_basic_kind() {
        0
    } else if kind.equip_slot().is_some() {
        2
    } else {
        1 // trick / delayed trick
    }
}

pub(super) fn is_hearth_derivable_card_name(raw: &str) -> bool {
    let b = exact_base_name(raw);
    matches!(
        b,
        "hsfashu_buwendingyibian"
            | "hsjixie_zhadan"
            | "hsqizhou_feng"
            | "hsqizhou_shui"
            | "hsqizhou_huo"
            | "hsqizhou_tu"
            | "hsqingyu_feibiao"
            | "hsqingyu_zhanfang"
            | "hsqingyu_hufu"
            | "hsqingyu_shandian"
            | "hsqingyu_zhao"
            | "hsdusu_xueji"
            | "hsdusu_kuyecao"
            | "hsdusu_huangxuecao"
            | "hsdusu_huoyanhua"
            | "hsdusu_shinancao"
            | "hsshenqi_nengliangzhiguang"
            | "hsshenqi_kongbusangzhong"
            | "hsshenqi_morijingxiang"
            | "hsmengjing_feicuiyoulong"
            | "hsmengjing_suxing"
            | "hsmengjing_mengye"
            | "hsmengjing_mengjing"
            | "hsmengjing_huanxiaojiemei"
            | "hszuzhou_nvwudeganguo"
            | "hszuzhou_nvwudepingguo"
            | "hszuzhou_nvwudexuetu"
            | "hszuzhou_wushushike"
            | "hszuzhou_guhuo"
    )
}

pub(super) fn is_hearth_vanish_card_name(raw: &str) -> bool {
    let b = exact_base_name(raw);
    b.starts_with("hsfashu_")
        || b.starts_with("hstianqi_")
        || b.starts_with("hsjixie_")
        || b.starts_with("hsqizhou_")
        || b.starts_with("hsqingyu_")
        || b.starts_with("hsdusu_")
        || b.starts_with("hsbaowu_")
        || b.starts_with("hsshenqi_")
        || b.starts_with("hsmengjing_")
        || b.starts_with("hszuzhou_")
        || b == "hsyaoshui"
        || b == "hsnitai_card"
}

pub(super) fn is_hearth_token_card(raw: &str) -> bool {
    raw.starts_with("hs_") || raw.starts_with("du_") || is_hearth_vanish_card_name(raw)
}

pub(super) fn is_du_card_hearth(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, cid: CardId) -> bool {
    let name = catalog.cards.get(cid as usize).map(|c| c.name).unwrap_or("");
    exact_base_name(name) == "du"
        || name == "毒"
        || flag(core, owner, &format!("is_du_card:{cid}")) > 0
}

/// 从 `detached_cards` 取出第一张匹配谓词的供应实体；不存在则退回弃牌堆（绝不重复分配同一物理牌）。
pub(super) fn pop_detached_matching(
    core: &mut GameStateCore,
    mut pred: impl FnMut(CardId) -> bool,
) -> Option<CardId> {
    if let Some(pos) = core.detached_cards.iter().position(|&(c, _)| pred(c)) {
        return Some(core.detached_cards.remove(pos).0);
    }
    None
}

pub(super) fn spawn_card_by_base(ctx: &mut Ctx<'_>, base: &str) -> Option<CardId> {
    let catalog = ctx.catalog;
    if let Some(c) = pop_detached_matching(ctx.core, |cid| {
        exact_base_name(catalog.cards[cid as usize].name) == base
    }) {
        return Some(c);
    }
    let hs_pref = format!("hs_{base}");
    if let Some(c) = pop_detached_matching(ctx.core, |cid| {
        exact_base_name(catalog.cards[cid as usize].name) == hs_pref
    }) {
        return Some(c);
    }
    if let Some(c) = pop_detached_matching(ctx.core, |cid| {
        base_card_name(catalog.cards[cid as usize].name) == base
    }) {
        return Some(c);
    }
    None
}

pub(super) fn spawn_card_by_kind(ctx: &mut Ctx<'_>, kind: CardKind) -> Option<CardId> {
    let catalog = ctx.catalog;
    if let Some(c) = pop_detached_matching(ctx.core, |cid| {
        let def = &catalog.cards[cid as usize];
        def.kind == kind && exact_base_name(def.name) != "du"
    }) {
        return Some(c);
    }
    if let Some(pos) = ctx.core.deck.iter().rposition(|&cid| {
        let def = &catalog.cards[cid as usize];
        def.kind == kind && exact_base_name(def.name) != "du"
    }) {
        return Some(ctx.core.deck.remove(pos));
    }
    if let Some(pos) = ctx.core.discard.iter().rposition(|&cid| {
        let def = &catalog.cards[cid as usize];
        def.kind == kind && exact_base_name(def.name) != "du"
    }) {
        return Some(ctx.core.discard.remove(pos));
    }
    None
}

/// 将结算完成的牌移至弃牌堆，若为炉石 `vanish: true` 或供应衍生牌且需消散则入 `detached_cards`。
pub(super) fn finish_table_card_hearth(ctx: &mut Ctx<'_>, card: CardId, vanish: bool) {
    ctx.core.table.retain(|&x| x != card);
    if vanish && is_hearth_token_card(ctx.catalog.cards[card as usize].name) {
        if !ctx.core.detached_cards.iter().any(|&(c, _)| c == card) {
            ctx.core
                .detached_cards
                .push((card, crate::event::DetachedKind::Destroyed));
        }
    } else {
        ctx.to_discard(card);
    }
}

pub(super) fn equip_card_to_player(ctx: &mut Ctx<'_>, target: PlayerId, card: CardId) {
    let Some(slot) = ctx.catalog.card_kind(card).equip_slot() else {
        return;
    };
    let replaced = ctx
        .core
        .player(target)
        .equips
        .iter()
        .copied()
        .find(|&e| ctx.catalog.cards[e as usize].kind.equip_slot() == Some(slot));
    if let Some(old) = replaced {
        ctx.core.player_mut(target).equips.retain(|&x| x != old);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, old);
        finish_table_card_hearth(ctx, old, true);
    }
    ctx.core.player_mut(target).equips.push(card);
    crate::kernel::grant_equip_skills(ctx, target, card);
    ctx.log(LogEvent::Equipped {
        player: target,
        card,
        replaced,
    });
}

pub(super) fn gain_card_to_hand(ctx: &mut Ctx<'_>, owner: PlayerId, card: CardId) {
    set_flag(ctx.core, owner, &format!("hs_wxuying_was_shadow:{card}"), 0);
    if !ctx.core.player(owner).hand.contains(&card) {
        ctx.core.player_mut(owner).hand.push(card);
    }
    ctx.log(LogEvent::CardsDrawn {
        player: owner,
        cards: vec![card],
    });
}

pub(super) fn discard_card_from_zone(ctx: &mut Ctx<'_>, seat: PlayerId, card: CardId) {
    if ctx.core.player(seat).hand.contains(&card) {
        ctx.take_hand_card(seat, card);
    } else if ctx.core.player(seat).equips.contains(&card) {
        ctx.core.player_mut(seat).equips.retain(|&x| x != card);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, seat, card);
    } else if ctx.core.player(seat).judge.contains(&card) {
        ctx.core.player_mut(seat).judge.retain(|&x| x != card);
    }
    finish_table_card_hearth(ctx, card, true);
}

pub(super) fn pick_random_hearth_derivable(ctx: &mut Ctx<'_>, count: usize) -> Vec<CardId> {
    let mut pool: Vec<CardId> = ctx
        .core
        .detached_cards
        .iter()
        .map(|&(c, _)| c)
        .filter(|&cid| is_hearth_derivable_card_name(ctx.catalog.cards[cid as usize].name))
        .collect();
    // Deduplicate by canonical base name so discovery offers distinct card types when possible
    let mut unique: Vec<CardId> = Vec::new();
    for cid in pool.drain(..) {
        let base = exact_base_name(ctx.catalog.cards[cid as usize].name);
        if !unique
            .iter()
            .any(|&u| exact_base_name(ctx.catalog.cards[u as usize].name) == base)
        {
            unique.push(cid);
        }
    }
    let mut picked = Vec::new();
    while !unique.is_empty() && picked.len() < count {
        let idx = ctx.core.rng.gen_range(unique.len() as u32) as usize;
        picked.push(unique.swap_remove(idx));
    }
    picked
}

pub(super) fn rng_pick<T: Copy>(ctx: &mut Ctx<'_>, slice: &[T]) -> Option<T> {
    if slice.is_empty() {
        return None;
    }
    let idx = ctx.core.rng.gen_range(slice.len() as u32) as usize;
    slice.get(idx).copied()
}

pub(super) fn rng_pick_mut<T>(ctx: &mut Ctx<'_>, vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        return None;
    }
    let idx = ctx.core.rng.gen_range(vec.len() as u32) as usize;
    Some(vec.swap_remove(idx))
}

pub(super) fn change_max_hp(ctx: &mut Ctx<'_>, seat: PlayerId, delta: i32) {
    let cur = ctx.core.player(seat).max_hp as i32;
    let next = (cur + delta).max(1) as u32;
    ctx.core.player_mut(seat).max_hp = next;
    if ctx.core.player(seat).hp > next as i32 {
        ctx.core.player_mut(seat).hp = next as i32;
    }
}

pub(super) fn nature_for_kind(kind: CardKind) -> DamageNature {
    match kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    }
}

pub(super) fn take_he_card(ctx: &mut Ctx<'_>, seat: PlayerId, card: CardId) {
    if ctx.core.player(seat).hand.contains(&card) {
        ctx.take_hand_card(seat, card);
    } else if ctx.core.player(seat).equips.contains(&card) {
        ctx.core.player_mut(seat).equips.retain(|&x| x != card);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, seat, card);
    }
}

pub(super) fn take_he_card_as(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    as_kind: CardKind,
) {
    take_he_card(ctx, owner, card);
    ctx.core.table.push(card);
    ctx.log(LogEvent::CardPlayed {
        card,
        player: owner,
        targets: targets.clone(),
        presentation: None,
    });
    ctx.emit(EventData::CardUsed(crate::machine::card_use_with_nature(
        card,
        owner,
        targets,
        nature_for_kind(as_kind),
        Some(as_kind),
    )));
}

pub(super) fn emit_hearth_virtual(
    ctx: &mut Ctx<'_>,
    source: PlayerId,
    as_kind: CardKind,
    targets: Vec<PlayerId>,
) {
    let nature = nature_for_kind(as_kind);
    if !ctx.core.deck.is_empty() {
        let carrier = ctx.core.deck.remove(0);
        ctx.core.player_mut(source).hand.push(carrier);
        ctx.emit(EventData::CardUsed(crate::machine::card_use_with_nature(
            carrier,
            source,
            targets,
            nature,
            Some(as_kind),
        )));
    } else {
        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
            card: None,
            player: source,
            targets,
            nature,
            use_as: Some(as_kind),
            cancelled: false,
        }));
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, skill_name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(skill_name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn in_attack_range_hearth(
    core: &GameStateCore,
    catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
) -> bool {
    a != b
        && core.player(a).alive
        && core.player(b).alive
        && core.computed_distance(catalog, a, b) <= core.attack_range(catalog, a)
}

pub(super) fn is_delayed_trick_hearth(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::Guiyoujie
    )
}

pub(super) fn is_hearth_unique_equip(catalog: &Catalog, cid: CardId) -> bool {
    let def = &catalog.cards[cid as usize];
    let exact = exact_base_name(def.name);
    exact.starts_with("hstianqi_")
        || exact.starts_with("gjyuheng")
        || matches!(
            def.kind,
            CardKind::WuxingPan
                | CardKind::Gjyuheng
                | CardKind::Xuanyuanjian
                | CardKind::Lianyaohu
                | CardKind::Qiankundai
                | CardKind::Nvwashi
                | CardKind::Shennongding
                | CardKind::Kunlunjingc
                | CardKind::Donghuangzhong
                | CardKind::Kongdongyin
                | CardKind::Guilingzhitao
                | CardKind::Sifeizhenmian
                | CardKind::Shuchui
                | CardKind::Sadengjinhuan
                | CardKind::Haotianta
                | CardKind::Yuruyi
                | CardKind::Yuchanqian
                | CardKind::Yuchankun
                | CardKind::Yuchanzhen
                | CardKind::Yuchanxun
                | CardKind::Yuchankan
                | CardKind::Yuchanli
                | CardKind::Yuchangen
                | CardKind::Yuchandui
                | CardKind::Shencaojie
                | CardKind::Qinglongzhigui
                | CardKind::Zhuquezhizhang
                | CardKind::Baishouzhihu
                | CardKind::Xuanwuzhihuang
                | CardKind::Huanglinzhicong
                | CardKind::Cangchizhibi
        )
}

pub(super) fn is_hearth_derived_weapon(catalog: &Catalog, cid: CardId) -> bool {
    let exact = exact_base_name(catalog.cards[cid as usize].name);
    exact.starts_with("hsqingyu_") || exact.starts_with("hstianqi_")
}

pub(super) fn pop_random_detached_weapon(ctx: &mut Ctx<'_>) -> Option<CardId> {
    let eligible = |c: CardId| {
        ctx.catalog.card_kind(c).equip_slot() == Some(crate::catalog::EquipSlot::Weapon)
            && !is_hearth_unique_equip(ctx.catalog, c)
            && !is_hearth_derived_weapon(ctx.catalog, c)
    };
    let candidates: Vec<usize> = ctx
        .core
        .detached_cards
        .iter()
        .enumerate()
        .filter(|(_, &(c, _))| eligible(c))
        .map(|(idx, _)| idx)
        .collect();
    if !candidates.is_empty() {
        let pick = ctx.core.rng.gen_range(candidates.len() as u32) as usize;
        let idx = candidates[pick];
        return Some(ctx.core.detached_cards.remove(idx).0);
    }
    let recycled: Vec<usize> = ctx
        .core
        .discard
        .iter()
        .enumerate()
        .filter(|(_, &c)| {
            let def = &ctx.catalog.cards[c as usize];
            def.off_deck && def.name.starts_with("hs_") && eligible(c)
        })
        .map(|(idx, _)| idx)
        .collect();
    if !recycled.is_empty() {
        let pick = ctx.core.rng.gen_range(recycled.len() as u32) as usize;
        let idx = recycled[pick];
        return Some(ctx.core.discard.remove(idx));
    }
    None
}

pub(super) fn hearth_sha_can_play(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    let unlimited = !core.turn.sha_used_play_phase
        || super::l10_reguhuo::has_unlimited_sha(core, catalog, owner);
    unlimited
        && alive_others(core, owner)
            .into_iter()
            .any(|p| in_attack_range_hearth(core, catalog, owner, p))
}

pub(super) fn hearth_jiu_can_play(core: &GameStateCore, _catalog: &Catalog, owner: PlayerId) -> bool {
    core.player(owner).alive && !core.turn.jiu_used
}

pub(super) fn hearth_tao_can_play(core: &GameStateCore, _catalog: &Catalog, owner: PlayerId) -> bool {
    core.player(owner).alive && core.player(owner).hp < core.player(owner).max_hp as i32
}

pub(super) fn collect_hearth_gainable_skills(
    catalog: &Catalog,
    exclude_name: &str,
) -> Vec<crate::ids::SkillDefId> {
    let mut pool = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let is_excluded_name = |sk: &str| -> bool {
        sk.is_empty()
            || sk == exclude_name
            || sk.starts_with('_')
            || sk.ends_with('2')
            || sk.ends_with('3')
            || sk.ends_with("_lose")
            || sk.starts_with("ysjqisha_")
            || sk.starts_with("yushou_")
            || sk.starts_with("hstianqi_")
            || sk.starts_with("boss_")
            || sk.starts_with("tuteng")
            || sk.starts_with("hsmengjing_")
            || matches!(
                sk,
                "jijiang"
                    | "激将"
                    | "hujia"
                    | "护驾"
                    | "jiuyuan"
                    | "救援"
                    | "xueyi"
                    | "血裔"
                    | "huangtian"
                    | "黄天"
                    | "zhiba"
                    | "制霸"
                    | "shichou"
                    | "ruoyu"
                    | "若愚"
                    | "weidi"
                    | "songwei"
                    | "xiongyi"
                    | "baonve"
                    | "shanshen"
                    | "hunzi"
                    | "魂姿"
                    | "zaoxian"
                    | "凿险"
                    | "zili"
                    | "自立"
                    | "zhiji"
                    | "志继"
                    | "baiyin"
                    | "拜印"
                    | "wuyong"
                    | "武勇"
                    | "benghuai"
                    | "崩坏"
                    | "fuyin"
                    | "伏荫"
                    | "maoxian"
                    | "jinhua"
                    | "szbianshen"
                    | "hsbaowu_cangbaotu"
                    | "hsbaowu_huangjinyuanhou"
            )
    };
    let mut consider = |general_name: &str, raw_sym: Option<&str>, skills: &[&str]| {
        if general_name.starts_with("boss_")
            || general_name.starts_with("gz_jun")
            || raw_sym.is_some_and(|s| s.starts_with("boss_") || s.starts_with("gz_jun"))
        {
            return;
        }
        for &sk in skills {
            if is_excluded_name(sk) {
                continue;
            }
            if let Some(def) = catalog.skill_by_name(sk) {
                if seen.insert(def) {
                    pool.push(def);
                }
            }
        }
    };
    for g in &catalog.standard_generals {
        consider(
            g.name,
            g.original_source.as_ref().map(|o| o.raw_symbol),
            g.skills,
        );
    }
    for g in &catalog.extension_generals {
        consider(g.name, None, g.skills);
    }
    pool
}

pub(in crate::content::host) fn trigger_meta(
    name: &str,
    behavior: HostBehavior,
) -> Option<TriggerMeta> {
    match behavior {
        HostBehavior::U05HearthCards => u05_hearth_cards::trigger_meta_cards(name),
        HostBehavior::U05HearthBatchA => u05_hearth_a::trigger_meta_a(name),
        HostBehavior::U05HearthBatchB => u05_hearth_b::trigger_meta_b(name),
        HostBehavior::U05HearthBatchC => u05_hearth_c::trigger_meta_c(name),
        HostBehavior::U05HearthBatchD => u05_hearth_d::trigger_meta_d(name),
        HostBehavior::U05HearthBatchE => u05_hearth_e::trigger_meta_e(name),
        _ => None,
    }
}

pub(in crate::content::host) fn limits(name: &str, behavior: HostBehavior) -> SkillLimits {
    match behavior {
        HostBehavior::U05HearthBatchA => u05_hearth_a::limits_a(name),
        HostBehavior::U05HearthBatchB => u05_hearth_b::limits_b(name),
        HostBehavior::U05HearthBatchC => u05_hearth_c::limits_c(name),
        HostBehavior::U05HearthBatchD => u05_hearth_d::limits_d(name),
        HostBehavior::U05HearthBatchE => u05_hearth_e::limits_e(name),
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_u05_hearth(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05HearthCards => u05_hearth_cards::filter_cards(skill, ctx, owner, event),
        HostBehavior::U05HearthBatchA => u05_hearth_a::filter_a(skill, ctx, owner, event),
        HostBehavior::U05HearthBatchB => u05_hearth_b::filter_b(skill, ctx, owner, event),
        HostBehavior::U05HearthBatchC => u05_hearth_c::filter_c(skill, ctx, owner, event),
        HostBehavior::U05HearthBatchD => u05_hearth_d::filter_d(skill, ctx, owner, event),
        HostBehavior::U05HearthBatchE => u05_hearth_e::filter_e(skill, ctx, owner, event),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u05_hearth(
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
    let eligible = match skill.behavior {
        HostBehavior::U05HearthBatchA => u05_hearth_a::activate_a(skill, ctx, owner),
        HostBehavior::U05HearthBatchB => u05_hearth_b::activate_b(skill, ctx, owner),
        HostBehavior::U05HearthBatchC => u05_hearth_c::activate_c(skill, ctx, owner),
        HostBehavior::U05HearthBatchD => u05_hearth_d::activate_d(skill, ctx, owner),
        HostBehavior::U05HearthBatchE => u05_hearth_e::activate_e(skill, ctx, owner),
        _ => None,
    };
    if eligible == Some(true) {
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

pub(in crate::content::host) fn has_phase_use_activation_hearth(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    match behavior {
        HostBehavior::U05HearthBatchA => u05_hearth_a::has_active_a(name),
        HostBehavior::U05HearthBatchB => u05_hearth_b::has_active_b(name),
        HostBehavior::U05HearthBatchC => u05_hearth_c::has_active_c(name),
        HostBehavior::U05HearthBatchD => u05_hearth_d::has_active_d(name),
        HostBehavior::U05HearthBatchE => u05_hearth_e::has_active_e(name),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_u05_hearth(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, card, targets) = match event {
        EventData::Damage(d) => (Some(d.target), d.card, Vec::new()),
        EventData::CardUsed(u) => (u.targets.first().copied(), u.card, u.targets.clone()),
        EventData::Equipped { player, card, .. } => (Some(*player), Some(*card), Vec::new()),
        EventData::Recover { player, .. } => (Some(*player), None, Vec::new()),
        EventData::Death { player, killer } => (Some(*player), killer.map(u32::from), Vec::new()),
        EventData::Dying { player, source } => (Some(*player), source.map(u32::from), Vec::new()),
        EventData::TurnBegin { player } | EventData::Phase { player, .. } => {
            (Some(*player), None, Vec::new())
        }
        EventData::CardsDiscarded { player, .. } | EventData::CardsLost { player, .. } => {
            (Some(*player), None, Vec::new())
        }
        _ => (None, None, Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_u05_hearth(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    match m.behavior {
        HostBehavior::U05HearthCards => u05_hearth_cards::step_hearth_cards(m, ctx),
        HostBehavior::U05HearthBatchA => u05_hearth_a::step_a(m, ctx),
        HostBehavior::U05HearthBatchB => u05_hearth_b::step_b(m, ctx),
        HostBehavior::U05HearthBatchC => u05_hearth_c::step_c(m, ctx),
        HostBehavior::U05HearthBatchD => u05_hearth_d::step_d(m, ctx),
        HostBehavior::U05HearthBatchE => u05_hearth_e::step_e(m, ctx),
        _ => Flow::Done,
    }
}

pub(in crate::content::host) fn meta(
    event: EventKind,
    timing: Timing,
    priority: i32,
    optional: bool,
    extra: Vec<EventKind>,
) -> TriggerMeta {
    TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: optional,
        optional,
        extra_events: extra,
    }
}

pub(in crate::content::host) fn make_damage(
    source: Option<PlayerId>,
    target: PlayerId,
    amount: i32,
    nature: DamageNature,
    card: Option<CardId>,
) -> DamageData {
    crate::machine::damage_data_with(source, target, amount, nature, card)
}
