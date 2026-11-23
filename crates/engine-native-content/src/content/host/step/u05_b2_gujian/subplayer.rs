use super::helpers::*;
use crate::catalog::EquipSlot;
use crate::ids::{CardId, PlayerId};
use crate::machine::Ctx;
use crate::state::SkillInstance;

pub(super) fn clean_yange_clone_flags(core: &mut crate::state::GameStateCore, card: CardId) {
    let key = format!("gj_yange_clone:{card}");
    if core.vm_aux.flags.get(&(0, key.clone())).copied().unwrap_or(0) > 0 {
        core.vm_aux.flags.remove(&(0, key));
        core.declared_kinds.remove(&card);
        core.declared_slots.remove(&card);
    }
}

fn sweep_yange_clone_piles(core: &mut crate::state::GameStateCore) {
    let to_clean: Vec<CardId> = core
        .discard
        .iter()
        .chain(core.deck.iter())
        .copied()
        .filter(|&c| {
            core.vm_aux
                .flags
                .get(&(0, format!("gj_yange_clone:{c}")))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .collect();
    for c in to_clean {
        clean_yange_clone_flags(core, c);
    }
}

fn equip_slot_to_u32(slot: EquipSlot) -> u32 {
    match slot {
        EquipSlot::Weapon => 0,
        EquipSlot::Armor => 1,
        EquipSlot::HorsePlus => 2,
        EquipSlot::HorseMinus => 3,
        EquipSlot::Treasure => 4,
    }
}

fn u32_to_equip_slot(v: u32) -> EquipSlot {
    match v {
        0 => EquipSlot::Weapon,
        1 => EquipSlot::Armor,
        2 => EquipSlot::HorsePlus,
        3 => EquipSlot::HorseMinus,
        _ => EquipSlot::Treasure,
    }
}

pub(super) fn stash_main_for_subplayer(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    active_flag: &str,
    prefix: &str,
) {
    let orig_hp = ctx.core.player(owner).hp;
    let orig_max = ctx.core.player(owner).max_hp;
    set_flag(ctx.core, owner, &format!("{prefix}_orig_max_hp"), u32::from(orig_max));
    set_flag(
        ctx.core,
        owner,
        &format!("{prefix}_orig_hp"),
        (orig_hp + 1000).max(0) as u32,
    );
    set_flag(ctx.core, owner, active_flag, 1);
    if let Some(gid) = ctx.core.player(owner).general_slots[0] {
        set_flag(ctx.core, owner, &format!("{prefix}_orig_gid"), u32::from(gid) + 1);
    }

    let orig_skills: Vec<(u16, SkillInstance)> = ctx
        .core
        .skills
        .iter()
        .filter(|(&(p, _), _)| p == owner)
        .map(|(&(_, d), inst)| (d, inst.clone()))
        .collect();
    set_flag(
        ctx.core,
        owner,
        &format!("{prefix}_orig_skills_len"),
        orig_skills.len() as u32,
    );
    for (idx, (def, inst)) in orig_skills.into_iter().enumerate() {
        let packed = u32::from(def)
            | (u32::from(inst.enabled) << 16)
            | (u32::from(inst.once_used) << 17)
            | (u32::from(inst.expires_end_of_turn) << 18)
            | (u32::from(inst.rounds) << 19);
        set_flag(ctx.core, owner, &format!("{prefix}_orig_skill:{idx}"), packed);
    }
    ctx.core.skills.retain(|&(p, _), _| p != owner);

    let orig_hand = std::mem::take(&mut ctx.core.player_mut(owner).hand);
    if !orig_hand.is_empty() {
        ctx.core
            .vm_aux
            .piles
            .insert((owner, format!("{prefix}_orig_hand")), orig_hand);
    }
    let orig_equips = std::mem::take(&mut ctx.core.player_mut(owner).equips);
    for &eq in &orig_equips {
        if let Some(slot) = ctx.core.declared_slots.remove(&eq) {
            set_flag(
                ctx.core,
                owner,
                &format!("{prefix}_orig_eq_slot:{eq}"),
                equip_slot_to_u32(slot) + 1,
            );
        }
    }
    if !orig_equips.is_empty() {
        ctx.core
            .vm_aux
            .piles
            .insert((owner, format!("{prefix}_orig_equips")), orig_equips);
    }
    let orig_off_court = std::mem::take(&mut ctx.core.player_mut(owner).off_court);
    if !orig_off_court.is_empty() {
        ctx.core
            .vm_aux
            .piles
            .insert((owner, format!("{prefix}_orig_off_court")), orig_off_court);
    }
}

pub(super) fn restore_subplayer(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    active_flag: &str,
    prefix: &str,
    clear_yange_clones: bool,
) -> bool {
    if flag(ctx.core, player, active_flag) == 0 {
        return false;
    }
    set_flag(ctx.core, player, active_flag, 0);

    // 弃置子将残余装备（须先撤技能再清除临时 clone/slot 定义）与手牌/离场牌，保留判定区
    let rem_equips = std::mem::take(&mut ctx.core.player_mut(player).equips);
    for eq in rem_equips {
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, player, eq);
        ctx.core.declared_slots.remove(&eq);
        clean_yange_clone_flags(ctx.core, eq);
        ctx.to_discard(eq);
    }
    let rem_hand = std::mem::take(&mut ctx.core.player_mut(player).hand);
    for c in rem_hand {
        set_flag(ctx.core, player, &format!("gj_zhenying_card:{c}"), 0);
        clean_yange_clone_flags(ctx.core, c);
        ctx.to_discard(c);
    }
    let rem_off = std::mem::take(&mut ctx.core.player_mut(player).off_court);
    for c in rem_off {
        clean_yange_clone_flags(ctx.core, c);
        ctx.to_discard(c);
    }
    if clear_yange_clones {
        sweep_yange_clone_piles(ctx.core);
    }

    ctx.core.skills.retain(|&(p, _), _| p != player);
    let gid_key = format!("{prefix}_orig_gid");
    let orig_gid = flag(ctx.core, player, &gid_key);
    if orig_gid > 0 {
        set_flag(ctx.core, player, &gid_key, 0);
        ctx.core.player_mut(player).general_slots[0] = Some((orig_gid - 1) as u16);
    }

    if let Some(orig_hand) = ctx
        .core
        .vm_aux
        .piles
        .remove(&(player, format!("{prefix}_orig_hand")))
    {
        ctx.core.player_mut(player).hand = orig_hand;
    }
    if let Some(orig_equips) = ctx
        .core
        .vm_aux
        .piles
        .remove(&(player, format!("{prefix}_orig_equips")))
    {
        for &eq in &orig_equips {
            let skey = format!("{prefix}_orig_eq_slot:{eq}");
            let sv = flag(ctx.core, player, &skey);
            set_flag(ctx.core, player, &skey, 0);
            if sv > 0 {
                ctx.core.declared_slots.insert(eq, u32_to_equip_slot(sv - 1));
            }
        }
        ctx.core.player_mut(player).equips = orig_equips.clone();
        for eq in orig_equips {
            crate::kernel::grant_equip_skills(ctx, player, eq);
        }
    }
    if let Some(orig_off) = ctx
        .core
        .vm_aux
        .piles
        .remove(&(player, format!("{prefix}_orig_off_court")))
    {
        ctx.core.player_mut(player).off_court = orig_off;
    }

    let sk_len_key = format!("{prefix}_orig_skills_len");
    let sk_len = flag(ctx.core, player, &sk_len_key);
    set_flag(ctx.core, player, &sk_len_key, 0);
    for idx in 0..sk_len {
        let key = format!("{prefix}_orig_skill:{idx}");
        let packed = flag(ctx.core, player, &key);
        set_flag(ctx.core, player, &key, 0);
        let def = (packed & 0xFFFF) as u16;
        let inst = SkillInstance {
            owner: player,
            enabled: ((packed >> 16) & 1) != 0,
            once_used: ((packed >> 17) & 1) != 0,
            expires_end_of_turn: ((packed >> 18) & 1) != 0,
            rounds: ((packed >> 19) & 0xFF) as u8,
        };
        ctx.insert_skill((player, def), inst);
    }

    let max_key = format!("{prefix}_orig_max_hp");
    let hp_key = format!("{prefix}_orig_hp");
    let orig_max = flag(ctx.core, player, &max_key);
    let orig_hp = flag(ctx.core, player, &hp_key);
    if orig_max > 0 {
        ctx.core.player_mut(player).max_hp = orig_max as u32;
        set_flag(ctx.core, player, &max_key, 0);
    }
    if orig_hp > 0 {
        ctx.core.player_mut(player).hp = orig_hp as i32 - 1000;
        set_flag(ctx.core, player, &hp_key, 0);
    }
    ctx.core.player_mut(player).dying = false;
    super::sync_gujian_owner_flags(ctx.core, ctx.catalog, player);
    true
}

pub(super) fn restore_yange_subplayer(ctx: &mut Ctx<'_>, player: PlayerId) -> bool {
    restore_subplayer(ctx, player, "gj_yange_active", "gj_yange", true)
}

pub(super) fn restore_humeng_subplayer(ctx: &mut Ctx<'_>, player: PlayerId) -> bool {
    restore_subplayer(ctx, player, "gj_humeng_sub_active", "gj_humeng", false)
}

pub(super) fn enter_humeng_subplayer(ctx: &mut Ctx<'_>, owner: PlayerId) {
    stash_main_for_subplayer(ctx, owner, "gj_humeng_sub_active", "gj_humeng");
    if let Some(sub) = general_id_by_name(ctx.catalog, "gjqt_yanjiaxieyi") {
        ctx.core.player_mut(owner).general_slots[0] = Some(sub);
    }
    ctx.core.player_mut(owner).max_hp = 2;
    ctx.core.player_mut(owner).hp = 2;
    ctx.core.player_mut(owner).dying = false;
    if let Some(def) = ctx.catalog.skill_by_name("xianju") {
        ctx.insert_skill(
            (owner, def),
            SkillInstance {
                owner,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
    super::sync_gujian_owner_flags(ctx.core, ctx.catalog, owner);
    ctx.start_draw(owner, 2);
}

fn clone_card_like(ctx: &mut Ctx<'_>, template: CardId) -> Option<CardId> {
    let base_tmpl = ctx.catalog.resolved_base_card_id(ctx.core, template);
    let tmpl = ctx.catalog.cards.get(base_tmpl as usize)?;
    let t_kind = ctx.catalog.resolved_card_kind(ctx.core, template);
    let t_name = tmpl.name;
    let t_suit = tmpl.suit;
    let t_rank = tmpl.rank;
    let t_slot = ctx.catalog.card_slot_of(ctx.core, template);
    let pick_from = |pool: &mut Vec<CardId>, pred: &dyn Fn(&crate::catalog::CardDef) -> bool| -> Option<CardId> {
        let pos = pool.iter().rposition(|&c| pred(&ctx.catalog.cards[c as usize]))?;
        Some(pool.remove(pos))
    };
    if let Some(c) = pick_from(&mut ctx.core.deck, &|d| {
        d.kind == t_kind && d.name == t_name && d.suit == t_suit && d.rank == t_rank
    })
    .or_else(|| {
        pick_from(&mut ctx.core.discard, &|d| {
            d.kind == t_kind && d.name == t_name && d.suit == t_suit && d.rank == t_rank
        })
    })
    .or_else(|| {
        pick_from(&mut ctx.core.deck, &|d| {
            d.kind == t_kind && d.name == t_name && d.suit == t_suit
        })
    })
    .or_else(|| {
        pick_from(&mut ctx.core.discard, &|d| {
            d.kind == t_kind && d.name == t_name && d.suit == t_suit
        })
    })
    .or_else(|| {
        pick_from(&mut ctx.core.deck, &|d| d.kind == t_kind && d.name == t_name)
    })
    .or_else(|| {
        pick_from(&mut ctx.core.discard, &|d| d.kind == t_kind && d.name == t_name)
    }) {
        clean_yange_clone_flags(ctx.core, c);
        return Some(c);
    }
    // 牌堆/弃堆无同牌定义（如唯一防具仁王盾或全部桃已在场）：借用载体牌并写入模板定义，离场/退出恢复
    let carrier = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop())?;
    clean_yange_clone_flags(ctx.core, carrier);
    set_flag(ctx.core, 0, &format!("gj_yange_clone:{carrier}"), base_tmpl + 1);
    ctx.core.declared_kinds.insert(carrier, t_kind);
    if let Some(slot) = t_slot {
        ctx.core.declared_slots.insert(carrier, slot);
    }
    Some(carrier)
}

pub(super) fn enter_yange_subplayer(ctx: &mut Ctx<'_>, owner: PlayerId, t: PlayerId) {
    stash_main_for_subplayer(ctx, owner, "gj_yange_active", "gj_yange");

    let stock_defs: Vec<u16> = ctx
        .core
        .player(t)
        .general_slots
        .into_iter()
        .flatten()
        .filter_map(|g| ctx.catalog.general_meta(g))
        .flat_map(|m| m.2)
        .filter_map(|sk| ctx.catalog.skill_by_name(sk))
        .collect();
    for def in stock_defs {
        ctx.insert_skill(
            (owner, def),
            SkillInstance {
                owner,
                enabled: true,
                once_used: false,
                expires_end_of_turn: true,
                rounds: 0,
            },
        );
    }

    let t_hand = ctx.core.player(t).hand.clone();
    for tc in t_hand {
        if let Some(c) = clone_card_like(ctx, tc) {
            ctx.core.player_mut(owner).hand.push(c);
        }
    }
    let t_equips = ctx.core.player(t).equips.clone();
    for te in t_equips {
        let Some(slot) = ctx.catalog.card_slot_of(ctx.core, te) else {
            continue;
        };
        if !crate::rvm_ops::has_enabled_equip_slot(ctx.core, owner, slot)
            || ctx
                .core
                .player(owner)
                .equips
                .iter()
                .any(|&q| ctx.catalog.card_slot_of(ctx.core, q) == Some(slot))
        {
            continue;
        }
        if let Some(ec) = clone_card_like(ctx, te) {
            ctx.core.declared_slots.insert(ec, slot);
            ctx.core.player_mut(owner).equips.push(ec);
            crate::kernel::grant_equip_skills(ctx, owner, ec);
        }
    }

    ctx.core.player_mut(owner).max_hp = ctx.core.player(t).max_hp;
    ctx.core.player_mut(owner).hp = ctx.core.player(t).hp;
    super::sync_gujian_owner_flags(ctx.core, ctx.catalog, owner);
}
