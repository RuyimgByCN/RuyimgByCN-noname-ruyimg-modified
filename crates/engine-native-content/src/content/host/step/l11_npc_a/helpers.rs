//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 共有辅助与生命周期钩子（`helpers.rs`）。
//!
//! 承载标志/堆存取、双面 `dualside` 切换、`reqizhou` 装备花色联动、`CardUsed`/`CardsLost`
//! 生命周期状态跟踪、弃牌统计以及 `dcmoukui` / `nzry_shicai` / `rexingluan` 延迟派生压栈。

use super::l11_common::*;
use super::l11_skills_b_asks::can_receive_moved_card;
use super::*;
use super::lifecycle::catalog_has_npc_a;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::ChoiceOption;
use crate::event::{EventData, LostOrigin};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::{GameStateCore, SkillInstance};

#[inline]
pub(super) fn inc_flag(core: &mut GameStateCore, p: PlayerId, key: &str, delta: u32) -> u32 {
    let next = flag(core, p, key).saturating_add(delta);
    set_flag(core, p, key, next);
    next
}

#[inline]
pub(super) fn is_unblocked_npc_a(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    is_unblocked_l11_owner(state, catalog, owner) && !catalog.is_xj_fengyin_sealed(state, owner)
}

#[inline]
pub(super) fn is_locked_npc_a(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    is_locked_unblocked_l11_owner(state, catalog, owner)
}

#[inline]
pub(super) fn has_npc_a_skill(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    name: &str,
) -> bool {
    has_named_skill(state, catalog, owner, name)
        && state.skills.iter().any(|((o, d), inst)| {
            *o == owner && inst.enabled && catalog.skills[*d as usize].name == name
        })
}

#[inline]
pub(super) fn card_suit(catalog: &Catalog, core: &GameStateCore, owner: PlayerId, c: CardId) -> Suit {
    catalog.effective_card_suit(core, owner, c)
}

#[inline]
pub(super) fn is_red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

#[inline]
pub(super) fn is_red_card(catalog: &Catalog, core: &GameStateCore, owner: PlayerId, c: CardId) -> bool {
    is_red_suit(card_suit(catalog, core, owner, c))
}

#[inline]
pub(super) fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1 << 0,
        Suit::Heart => 1 << 1,
        Suit::Club => 1 << 2,
        Suit::Diamond => 1 << 3,
    }
}

/// 卡牌类别分类（`1 = 基本`, `2 = 锦囊`, `3 = 装备`）。
#[inline]
pub(super) fn card_category(kind: CardKind) -> u8 {
    if kind.is_basic_kind() {
        1
    } else if kind.equip_slot().is_some() {
        3
    } else {
        2
    }
}

#[inline]
pub(super) fn is_ordinary_trick_kind(kind: CardKind) -> bool {
    kind.is_trick_kind()
        && !matches!(
            kind,
            CardKind::Le
                | CardKind::Bing
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Guiyoujie
        )
}

#[inline]
pub(super) fn is_single_target_basic_or_trick(kind: CardKind) -> bool {
    kind.is_basic_kind()
        || (kind.is_trick_kind()
            && !matches!(
                kind,
                CardKind::Le
                    | CardKind::Bing
                    | CardKind::ShanDian
                    | CardKind::HongShui
                    | CardKind::Guiyoujie
                    | CardKind::Wugu
                    | CardKind::Taoyuan
                    | CardKind::SavageAttack
                    | CardKind::ArcheryAttack
                    | CardKind::Xianluhui
                    | CardKind::Chiyuxi
                    | CardKind::Jingleishan
                    | CardKind::Liufengsan
                    | CardKind::Shezhanqunru
            ))
}

#[inline]
pub(super) fn has_used_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.skills.get(&(owner, def)).is_some_and(|i| i.once_used))
}

pub(super) fn mark_skill_used(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = true;
        }
    }
}

pub(super) fn pop_deck_reshuffle(ctx: &mut Ctx<'_>) -> Option<CardId> {
    if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
        let mut pile = std::mem::take(&mut ctx.core.discard);
        ctx.core.rng.shuffle(&mut pile);
        ctx.core.deck = pile;
        (ctx.catalog.hooks.on_deck_washed)(ctx);
    }
    ctx.core.deck.pop()
}

pub(super) fn take_from_table_discard_or_equip(
    ctx: &mut Ctx<'_>,
    cid: CardId,
) -> bool {
    if let Some(pos) = ctx.core.table.iter().position(|&c| c == cid) {
        ctx.core.table.remove(pos);
        return true;
    }
    if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
        ctx.core.discard.remove(pos);
        return true;
    }
    let seats = ctx.core.players.len() as PlayerId;
    for seat in 0..seats {
        if ctx.core.player(seat).equips.contains(&cid) {
            ctx.take_owned_card(seat, cid);
            return true;
        }
    }
    false
}

pub(super) fn put_on_top_of_deck(ctx: &mut Ctx<'_>, cid: CardId) {
    ctx.core.deck.push(cid);
}

pub(super) fn put_on_bottom_of_deck(ctx: &mut Ctx<'_>, cid: CardId) {
    ctx.core.deck.insert(0, cid);
}

pub(super) fn equip_card_direct(
    ctx: &mut Ctx<'_>,
    target: PlayerId,
    cid: CardId,
) {
    let slot = ctx.catalog.card_slot_of(ctx.core, cid);
    let mut replaced = None;
    if let Some(sl) = slot {
        if sl != EquipSlot::Treasure || !has_named_skill(ctx.core, ctx.catalog, target, "_txhj_equip5") {
            let old = ctx
                .core
                .player(target)
                .equips
                .iter()
                .copied()
                .find(|&eq| ctx.catalog.card_slot_of(ctx.core, eq) == Some(sl));
            if let Some(prev) = old {
                ctx.take_owned_card(target, prev);
                ctx.to_discard(prev);
                replaced = Some(prev);
            }
        }
    }
    ctx.core.player_mut(target).hand.push(cid);
    ctx.emit(EventData::Equipped {
        player: target,
        card: cid,
        replaced,
    });
}

pub(super) fn swap_equip_zones(ctx: &mut Ctx<'_>, a: PlayerId, b: PlayerId) {
    let ea: Vec<CardId> = ctx.core.player(a).equips.clone();
    let eb: Vec<CardId> = ctx.core.player(b).equips.clone();
    for &c in &ea {
        ctx.core.player_mut(a).equips.retain(|&x| x != c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, a, c);
    }
    for &c in &eb {
        ctx.core.player_mut(b).equips.retain(|&x| x != c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, b, c);
    }
    for &c in &eb {
        ctx.core.player_mut(a).equips.push(c);
        crate::kernel::grant_equip_skills(ctx, a, c);
    }
    for &c in &ea {
        ctx.core.player_mut(b).equips.push(c);
        crate::kernel::grant_equip_skills(ctx, b, c);
    }
    if !ea.is_empty() {
        ctx.note_cards_lost(a, &ea);
        ctx.emit(EventData::CardsLost {
            player: a,
            cards: ea.clone(),
            origins: vec![LostOrigin::Equip; ea.len()],
        });
    }
    if !eb.is_empty() {
        ctx.note_cards_lost(b, &eb);
        ctx.emit(EventData::CardsLost {
            player: b,
            cards: eb.clone(),
            origins: vec![LostOrigin::Equip; eb.len()],
        });
    }
    sync_reqizhou_skills(ctx.core, ctx.catalog, a);
    sync_reqizhou_skills(ctx.core, ctx.catalog, b);
}

pub(super) fn move_card_ej(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    cid: CardId,
) -> bool {
    if from == to || !can_receive_moved_card(ctx.core, ctx.catalog, from, to, cid) {
        return false;
    }
    if ctx.core.player(from).equips.contains(&cid) {
        ctx.take_owned_card(from, cid);
        ctx.core.player_mut(to).equips.push(cid);
        crate::kernel::grant_equip_skills(ctx, to, cid);
        sync_reqizhou_skills(ctx.core, ctx.catalog, from);
        sync_reqizhou_skills(ctx.core, ctx.catalog, to);
        return true;
    }
    if ctx.core.player(from).judge.contains(&cid) {
        ctx.take_owned_card(from, cid);
        ctx.core.player_mut(to).judge.push(cid);
        return true;
    }
    false
}

/// `mobile.js:13283` (`reqizhou` 奇洲): 按装备区不同花色数同步授予/回收
/// `reyingzi`(1)、`qixi`/`奇袭`(2)、`rexuanfeng`(3)。
pub(in crate::content::host) fn sync_reqizhou_skills(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) {
    let active = core.player(owner).alive && has_npc_a_skill(core, catalog, owner, "reqizhou");
    let mut suit_mask = 0u32;
    if active {
        for &c in &core.player(owner).equips {
            suit_mask |= suit_bit(card_suit(catalog, core, owner, c));
        }
    }
    let suits = suit_mask.count_ones();
    let specs: [(&str, Option<&str>, u32, u32); 3] = [
        ("reyingzi", None, 1, 1),
        ("qixi", Some("奇袭"), 2, 2),
        ("rexuanfeng", None, 4, 3),
    ];
    let mut owned_mask = flag(core, owner, "l11_reqizhou_owned");
    for (primary, fallback, bit, need) in specs {
        let Some(def) = catalog
            .skill_by_name(primary)
            .or_else(|| fallback.and_then(|f| catalog.skill_by_name(f)))
        else {
            continue;
        };
        let want = active && suits >= need;
        if want {
            if !core.has_skill(owner, def) {
                core.skills.insert(
                    (owner, def),
                    SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
                owned_mask |= bit;
            }
        } else if (owned_mask & bit) != 0 {
            core.skills.remove(&(owner, def));
            owned_mask &= !bit;
        }
    }
    set_flag(core, owner, "l11_reqizhou_owned", owned_mask);
}

fn find_general_index(catalog: &Catalog, name: &str) -> Option<(u16, Vec<&'static str>, u32)> {
    let total = catalog.standard_generals.len() + catalog.extension_generals.len();
    for idx in 0..total {
        if let Some((gname, _, skills, max_hp, _)) = catalog.general_meta(idx as u16) {
            if gname == name {
                return Some((idx as u16, skills, max_hp));
            }
        }
    }
    None
}

fn dualside_pair_for(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "txhj_muludawang" | "txhj_menghuo6" => Some(("txhj_muludawang", "txhj_menghuo6")),
        "txhj_mangyachang" | "txhj_menghuo2" => Some(("txhj_mangyachang", "txhj_menghuo2")),
        _ => None,
    }
}

/// `game.js:34262` (`dualside`): 翻面或阵亡时切换正/背面武将（或兜底翻面回满摸 2）。
pub(super) fn execute_dualside_switch(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    from_death: bool,
) -> bool {
    if !has_npc_a_skill(ctx.core, ctx.catalog, owner, "dualside")
        || flag(ctx.core, owner, "l11_dualside_over") > 0
    {
        return false;
    }
    let cur_slot = ctx.core.player(owner).general_slots[0];
    let cur_name = cur_slot.and_then(|g| ctx.catalog.general_meta(g).map(|(n, ..)| n));
    if let Some((front_n, back_n)) = cur_name.and_then(dualside_pair_for) {
        let on_back = cur_name == Some(back_n) || flag(ctx.core, owner, "l11_dualside_side") == 1;
        let next_name = if on_back { front_n } else { back_n };
        if let Some((next_idx, next_skills, def_max)) = find_general_index(ctx.catalog, next_name) {
            let cur_hp = ctx.core.player(owner).hp.max(1) as u32;
            let cur_max = ctx.core.player(owner).max_hp;
            if on_back {
                set_flag(ctx.core, owner, "l11_dualside_back_hp", cur_hp);
                set_flag(ctx.core, owner, "l11_dualside_back_max", cur_max);
                let restored_max = flag(ctx.core, owner, "l11_dualside_front_max").max(1);
                let restored_hp = flag(ctx.core, owner, "l11_dualside_front_hp")
                    .max(1)
                    .min(restored_max);
                ctx.core.player_mut(owner).max_hp = restored_max;
                ctx.core.player_mut(owner).hp = restored_hp as i32;
                set_flag(ctx.core, owner, "l11_dualside_side", 0);
            } else {
                set_flag(ctx.core, owner, "l11_dualside_front_hp", cur_hp);
                set_flag(ctx.core, owner, "l11_dualside_front_max", cur_max);
                let saved_max = flag(ctx.core, owner, "l11_dualside_back_max");
                let restored_max = if saved_max > 0 { saved_max } else { def_max };
                let saved_hp = flag(ctx.core, owner, "l11_dualside_back_hp");
                let restored_hp = if saved_hp > 0 {
                    saved_hp.min(restored_max)
                } else {
                    restored_max
                };
                ctx.core.player_mut(owner).max_hp = restored_max;
                ctx.core.player_mut(owner).hp = restored_hp as i32;
                set_flag(ctx.core, owner, "l11_dualside_side", 1);
            }
            if let Some(old_idx) = cur_slot {
                if let Some((_, _, old_skills, ..)) = ctx.catalog.general_meta(old_idx) {
                    for sk in old_skills {
                        if sk != "dualside" {
                            remove_named_skill(ctx, owner, sk);
                        }
                    }
                }
            }
            for sk in next_skills {
                ensure_perm_skill(ctx, owner, sk);
            }
            ctx.core.player_mut(owner).general_slots[0] = Some(next_idx);
            ctx.core.player_mut(owner).dying = false;
            ctx.core.player_mut(owner).death_cancelled = from_death;
            if from_death {
                set_flag(ctx.core, owner, "l11_dualside_over", 1);
            }
            crate::resolution::sync_trigger_owners(ctx.core, ctx.res);
            return true;
        }
    }
    // 独立挂载 `dualside` 无配对表项时的通用切换（保持回满至体力上限并摸 2 牌语义）
    let flipped = !ctx.core.player(owner).face_down;
    ctx.core.player_mut(owner).face_down = flipped;
    ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.max(1);
    ctx.core.player_mut(owner).hp = ctx.core.player(owner).max_hp as i32;
    ctx.core.player_mut(owner).dying = false;
    if from_death {
        ctx.core.player_mut(owner).death_cancelled = true;
        set_flag(ctx.core, owner, "l11_dualside_over", 1);
    }
    draw_direct_cards(ctx, owner, 2);
    true
}

/// `Death` 执行前钩子：由 `content/hooks.rs::on_death_before` 调用，拦截 `dualside` 首次致死切换形态。
pub(in crate::content::host) fn on_death_before_l11_npc_a(
    ctx: &mut Ctx<'_>,
    dying: PlayerId,
) -> bool {
    if !catalog_has_npc_a(ctx.catalog) {
        return false;
    }
    execute_dualside_switch(ctx, dying, true)
}

pub(in crate::content::host) fn push_npc_a_park(ctx: &mut Ctx<'_>, machine: HostMachine) {
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(machine),
    });
}

#[inline]
pub(super) fn wait_npc_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(0);
    set_flag(ctx.core, m.owner, "l11_npc_a_ask", ask_id);
    if let Some((observer, spec)) = super::asks::build_l11_npc_a_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        ctx.core
            .vm_aux
            .flags
            .remove(&(m.owner, "l11_npc_a_ask".to_string()));
        Flow::Done
    }
}

#[inline]
pub(super) fn clear_and_done(m: &mut HostMachine) -> Flow {
    m.card = None;
    Flow::Done
}

#[inline]
pub(super) fn clear_and_yield(m: &mut HostMachine, next: HostStage) -> Flow {
    m.card = None;
    m.stage = next;
    Flow::Yield
}

#[inline]
pub(super) fn answered_tag(ctx: &Ctx<'_>) -> Option<u8> {
    ctx.answer().iter().find_map(|o| match o {
        ChoiceOption::Tag(v) => Some(*v),
        _ => None,
    })
}

pub(super) fn emit_recover(ctx: &mut Ctx<'_>, player: PlayerId, amount: u32) {
    if amount > 0 && ctx.core.player(player).alive {
        ctx.recover(player, amount, None);
    }
}

pub(super) fn emit_turn_over(ctx: &mut Ctx<'_>, player: PlayerId) {
    if ctx.core.player(player).alive {
        ctx.emit(EventData::TurnOverBefore {
            player,
            cancelled: false,
        });
    }
}

pub(super) fn consume_he_card_for_virtual_use(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    cid: CardId,
) {
    if ctx.core.player(owner).hand.contains(&cid) {
        ctx.take_hand_card(owner, cid);
        ctx.to_discard(cid);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![cid],
            origins: vec![LostOrigin::Hand],
        });
    } else if ctx.core.player(owner).equips.contains(&cid) {
        ctx.take_owned_card(owner, cid);
        ctx.to_discard(cid);
    }
}

pub(super) fn finish_respond_park(ctx: &mut Ctx<'_>, owner: PlayerId, ok: bool) -> Flow {
    let fid = flag(ctx.core, owner, "u01_respond_fid");
    if fid != 0 {
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, "u01_respond_fid".to_string()));
        set_flag(ctx.core, owner, &format!("u01_respond_result:{fid}"), u32::from(ok));
    }
    Flow::Done
}

pub(super) fn jili_can_trigger(
    ctx: &crate::catalog::QueryCtx<'_>,
    owner: PlayerId,
    u: &crate::event::CardUseData,
) -> bool {
    if u.cancelled || u.player == owner || u.targets.contains(&owner) {
        return false;
    }
    let k = u.effective_kind(ctx.catalog);
    let red = u
        .card
        .is_some_and(|c| is_red_card(ctx.catalog, ctx.state, u.player, c));
    if !red || !(k.is_basic_kind() || is_ordinary_trick_kind(k)) {
        return false;
    }
    let effective_cname = if u.use_as.is_none() {
        u.card
            .and_then(|c| ctx.catalog.cards.get(c as usize))
            .map_or("", |d| crate::content::host::intrinsic::base_card_key(d.name))
    } else {
        ""
    };
    let plain_wuzhong = k == CardKind::Wuzhong
        && (u.use_as == Some(CardKind::Wuzhong)
            || u.card.is_some_and(|c| {
                !crate::content::host::intrinsic::is_named_wuzhong_unwrapped(ctx.catalog, c)
            }));
    let is_shihuawuqi = k == CardKind::Tao
        && u.card
            .and_then(|c| ctx.catalog.cards.get(c as usize))
            .is_some_and(|d| {
                crate::content::host::intrinsic::base_card_key(d.name).ends_with("shihuawuqi")
            });
    if is_shihuawuqi {
        return false;
    }
    if u.targets.is_empty() && !(plain_wuzhong || k == CardKind::Tao) {
        return false;
    }
    if matches!(
        k,
        CardKind::Shan
            | CardKind::Jiu
            | CardKind::Tianxianjiu
            | CardKind::Nullify
            | CardKind::Wugu
            | CardKind::Taoyuan
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Jiedao
            | CardKind::Shezhanqunru
            | CardKind::Xianluhui
            | CardKind::Chiyuxi
            | CardKind::Jingleishan
            | CardKind::Liufengsan
            | CardKind::Linghunzhihuo
            | CardKind::Yuansuhuimie
            | CardKind::Shijieshu
            | CardKind::Jinlianzhu
            | CardKind::Yangpijuan
            | CardKind::Dunpaigedang
            | CardKind::Xingjiegoutong
            | CardKind::Chuansongmen
            | CardKind::Guisheqi
            | CardKind::Shencaojie
    ) || matches!(k, CardKind::Shuigong if effective_cname != "txhj_shuiyan")
        || matches!(
            effective_cname,
            "txhj_jiuchiroulin"
                | "txhj_yiyi"
                | "yiyi"
                | "txhj_jiedao"
                | "txhj_shengdong"
                | "txhj_huoshaolianying"
                | "huoshaolianying"
                | "txzhaoshu_skill"
                | "txzhaoshu_global"
        )
    {
        return false;
    }
    if !ctx
        .catalog
        .ow_target_enabled(ctx.state, u.player, owner, u.card, k, u.nature)
    {
        return false;
    }
    let has_skill_name = |name: &str| {
        ctx.catalog
            .skill_by_name(name)
            .is_some_and(|d| ctx.state.has_skill(owner, d))
    };
    let owner_p = ctx.state.player(owner);
    let hej_nonempty =
        !owner_p.hand.is_empty() || !owner_p.equips.is_empty() || !owner_p.judge.is_empty();
    let target_ok = match effective_cname {
        "huogong" => !owner_p.hand.is_empty(),
        "txhj_zhijizhibi" => {
            let unseen_in_kingdoms = matches!(ctx.state.mode, crate::state::GameMode::Kingdoms)
                && owner_p.revealed_generals.iter().any(|&r| !r);
            !owner_p.hand.is_empty() || unseen_in_kingdoms
        }
        "txhj_yuanjiao" | "txhj_diaohu" | "txhj_caomu" | "txhj_wangmeizhike" | "txhj_shuiyan" => {
            true
        }
        _ => match k {
            CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
                !crate::kernel::sha_user_or_target_blocked(ctx.state, ctx.catalog, u.player, owner)
            }
            CardKind::Duel => {
                !(owner_p.hand.is_empty()
                    && (has_skill_name(crate::vocabulary::names::KONG_CHENG)
                        || has_skill_name("kongcheng")))
            }
            CardKind::Tao
            | CardKind::Hufu
            | CardKind::Dinvxuanshuang
            | CardKind::Caoyao
            | CardKind::Zhiliaobo => owner_p.hp < owner_p.max_hp as i32,
            CardKind::Snatch => {
                hej_nonempty
                    && !has_skill_name(crate::vocabulary::names::QIAN_XUN)
                    && !has_skill_name("qianxun")
            }
            CardKind::Dismantle | CardKind::Zhaomingdan | CardKind::Shujinsan => hej_nonempty,
            CardKind::Fudichouxin
            | CardKind::Toulianghuanzhu
            | CardKind::Caochuanjiejian => !owner_p.hand.is_empty(),
            CardKind::Chenhuodajie => !owner_p.hand.is_empty() || !owner_p.equips.is_empty(),
            CardKind::Wuzhong => plain_wuzhong,
            CardKind::Jihuocard => false,
            _ => true,
        },
    };
    if !target_ok {
        return false;
    }
    let eff_targets: &[PlayerId] = if u.targets.is_empty() {
        std::slice::from_ref(&u.player)
    } else {
        &u.targets
    };
    eff_targets.iter().any(|&t| {
        ctx.state.player(t).alive && ctx.state.computed_distance(ctx.catalog, t, owner) <= 1
    })
}
