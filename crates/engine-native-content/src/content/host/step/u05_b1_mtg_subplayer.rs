//! `lingyong` / `subplayer` 副将生命周期实现（`character/mtg/skill.js:34-76`、`noname/library/element/player.js` / `content.ts`、`noname/library/skill.js:1565-1625`）。

use super::l27_batch_a::{flag, grant_skill_named, set_flag};
use crate::catalog::Catalog;
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

fn has_skill_named(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, name: &str) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|d| core.has_skill(owner, d))
}

fn is_nosub_skill(name: &str) -> bool {
    matches!(name, "lingyong" | "nshuanxian")
}

fn general_names_match(catalog: &Catalog, a: u16, b: u16) -> bool {
    a == b
        || matches!(
            (catalog.general_meta(a), catalog.general_meta(b)),
            (Some((na, ..)), Some((nb, ..))) if na == nb
        )
}

pub(crate) fn has_active_subplayer(core: &GameStateCore, owner: PlayerId) -> bool {
    flag(core, owner, "u05_sub_active") > 0
}

pub(crate) fn active_subplayer_slots(core: &GameStateCore, owner: PlayerId) -> Vec<u32> {
    let cnt = flag(core, owner, "u05_lingyong_gen_count");
    (0..cnt)
        .filter(|i| flag(core, owner, &format!("u05_lingyong_gen:{i}")) > 0)
        .collect()
}

pub(crate) fn capture_lingyong_subplayer(ctx: &mut Ctx<'_>, owner: PlayerId, dead: PlayerId) {
    if !has_skill_named(ctx.core, ctx.catalog, owner, "lingyong")
        || has_skill_named(ctx.core, ctx.catalog, owner, "subplayer")
        || has_active_subplayer(ctx.core, owner)
        || ctx.catalog.is_tieji_sealed(ctx.core, owner)
    {
        return;
    }
    let Some(dead_gid) = ctx.core.player(dead).general_slots[0] else {
        return;
    };
    let owner_slots = ctx.core.player(owner).general_slots;
    if owner_slots
        .iter()
        .flatten()
        .any(|&og| general_names_match(ctx.catalog, og, dead_gid))
    {
        return;
    }
    let cnt = flag(ctx.core, owner, "u05_lingyong_gen_count");
    let mut hs = Vec::with_capacity(2);
    for _ in 0..2 {
        if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
            hs.push(ctx.draw_judge_card());
        }
    }
    ctx.core
        .vm_aux
        .piles
        .insert((owner, format!("u05_sub_hs:{cnt}")), hs);
    set_flag(
        ctx.core,
        owner,
        &format!("u05_lingyong_gen:{cnt}"),
        u32::from(dead_gid) + 1,
    );
    set_flag(ctx.core, owner, "u05_lingyong_gen_count", cnt + 1);
}

pub(crate) fn cleanup_lingyong_on_remove(
    core: &mut GameStateCore,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    let mut returned = Vec::new();
    match skill_name {
        "lingyong" if !has_active_subplayer(core, owner) => {
            let cnt = flag(core, owner, "u05_lingyong_gen_count");
            for i in 0..cnt {
                set_flag(core, owner, &format!("u05_lingyong_gen:{i}"), 0);
                if let Some(p) = core.vm_aux.piles.remove(&(owner, format!("u05_sub_hs:{i}"))) {
                    returned.extend(p);
                }
            }
            set_flag(core, owner, "u05_lingyong_gen_count", 0);
        }
        "subplayer" => {
            for key in ["u05_sub_main_hs", "u05_sub_main_es"] {
                if let Some(p) = core.vm_aux.piles.remove(&(owner, key.to_string())) {
                    returned.extend(p);
                }
            }
            set_flag(core, owner, "u05_sub_active", 0);
        }
        _ => {}
    }
    returned
}

pub(crate) fn enter_subplayer(ctx: &mut Ctx<'_>, owner: PlayerId, slot_idx: u32) -> bool {
    if has_active_subplayer(ctx.core, owner) {
        return false;
    }
    let g_val = flag(ctx.core, owner, &format!("u05_lingyong_gen:{slot_idx}"));
    if g_val == 0 {
        return false;
    }
    let sub_gid = (g_val - 1) as u16;
    let Some(sub_meta) = ctx.catalog.general_meta(sub_gid) else {
        return false;
    };
    let sub_skills: Vec<&'static str> = sub_meta
        .2
        .iter()
        .copied()
        .filter(|sk| !is_nosub_skill(sk))
        .collect();

    // 1. Stash main general identity, stats, skills, hand, equips (`_triggered = null` matching player.lose -> ui.special)
    let (main_hp, main_max_hp, main_hujia, main_slots, main_hand, main_equips) = {
        let p = ctx.core.player_mut(owner);
        let hs = std::mem::take(&mut p.hand);
        let es = std::mem::take(&mut p.equips);
        (p.hp, p.max_hp, p.hujia, p.general_slots, hs, es)
    };
    ctx.core
        .vm_aux
        .piles
        .insert((owner, "u05_sub_main_hs".to_string()), main_hand);
    ctx.core
        .vm_aux
        .piles
        .insert((owner, "u05_sub_main_es".to_string()), main_equips);
    set_flag(ctx.core, owner, "u05_sub_main_hp", (main_hp + 1000).max(0) as u32);
    set_flag(ctx.core, owner, "u05_sub_main_maxhp", main_max_hp);
    set_flag(ctx.core, owner, "u05_sub_main_hujia", u32::from(main_hujia));
    set_flag(
        ctx.core,
        owner,
        "u05_sub_main_g0",
        main_slots[0].map(|g| u32::from(g) + 1).unwrap_or(0),
    );
    set_flag(
        ctx.core,
        owner,
        "u05_sub_main_g1",
        main_slots[1].map(|g| u32::from(g) + 1).unwrap_or(0),
    );

    let saved: Vec<(u16, SkillInstance)> = ctx
        .core
        .skills
        .iter()
        .filter(|((o, _), _)| *o == owner)
        .map(|((_, d), inst)| (*d, inst.clone()))
        .collect();
    set_flag(ctx.core, owner, "u05_sub_main_sk_cnt", saved.len() as u32);
    for (idx, (def, inst)) in saved.iter().enumerate() {
        let encoded = u32::from(*def)
            | (u32::from(inst.enabled) << 16)
            | (u32::from(inst.once_used) << 17)
            | (u32::from(inst.expires_end_of_turn) << 18)
            | (u32::from(inst.rounds) << 19)
            | (1u32 << 31);
        set_flag(ctx.core, owner, &format!("u05_sub_main_sk:{idx}"), encoded);
        ctx.core.skills.remove(&(owner, *def));
    }

    // 2. Apply SubPlayer state: `[hp=1, maxHp=1, hujia=0]`, `subplayer` + non-`nosub` skills, `hs: 2`
    set_flag(ctx.core, owner, "u05_sub_active", slot_idx + 1);
    let sub_hs = ctx
        .core
        .vm_aux
        .piles
        .remove(&(owner, format!("u05_sub_hs:{slot_idx}")))
        .unwrap_or_default();
    {
        let p = ctx.core.player_mut(owner);
        p.hp = 1;
        p.max_hp = 1;
        p.hujia = 0;
        p.general_slots = [Some(sub_gid), None];
        p.hand = sub_hs;
    }
    let _ = grant_skill_named(ctx, owner, "subplayer");
    for sk in sub_skills {
        let _ = grant_skill_named(ctx, owner, sk);
    }
    true
}

pub(crate) fn exit_subplayer_permanent(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let active = flag(ctx.core, owner, "u05_sub_active");
    if active == 0 {
        return;
    }
    let slot_idx = active - 1;
    ctx.core.player_mut(owner).death_cancelled = true;
    for f in ctx.res.stack.iter().rev() {
        if let EventData::Damage(d) = &f.data {
            if d.target == owner {
                set_flag(
                    ctx.core,
                    owner,
                    &format!("subplayer_untrigger_dmg:{}", f.id),
                    1,
                );
                break;
            }
        }
    }

    // 1. Discard SubPlayer `he` (`_triggered = null`) & remove SubPlayer skills
    let (mut sub_hand, sub_equips) = {
        let p = ctx.core.player_mut(owner);
        (std::mem::take(&mut p.hand), std::mem::take(&mut p.equips))
    };
    for &c in &sub_equips {
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
    }
    sub_hand.extend(sub_equips);
    let mut sub_discards = Vec::new();
    for c in sub_hand {
        ctx.to_discard(c);
        sub_discards.push(c);
    }
    if !sub_discards.is_empty() {
        ctx.log(LogEvent::CardDiscarded {
            player: owner,
            cards: sub_discards,
        });
    }

    // Mark active slot 0 first so `on_skill_removed` knows permanent subplayer exit is in progress
    set_flag(ctx.core, owner, &format!("u05_lingyong_gen:{slot_idx}"), 0);
    set_flag(ctx.core, owner, "u05_sub_active", 0);

    let sub_defs: Vec<u16> = ctx
        .core
        .skills
        .keys()
        .filter(|(o, _)| *o == owner)
        .map(|(_, d)| *d)
        .collect();
    for def in sub_defs {
        let name = ctx.catalog.skill_name(def);
        ctx.core.skills.remove(&(owner, def));
        if name != "subplayer" {
            let extra = (ctx.catalog.hooks.on_skill_removed)(ctx.core, ctx.catalog, owner, name);
            for c in extra {
                ctx.to_discard(c);
            }
        }
        ctx.log(LogEvent::SkillToggled {
            player: owner,
            skill: name,
            enabled: false,
        });
    }

    // 2. Restore Main General identity, stats, skills, hand, equips
    let main_hp = flag(ctx.core, owner, "u05_sub_main_hp") as i32 - 1000;
    let main_max_hp = flag(ctx.core, owner, "u05_sub_main_maxhp").max(1);
    let main_hujia = flag(ctx.core, owner, "u05_sub_main_hujia") as u8;
    let g0 = flag(ctx.core, owner, "u05_sub_main_g0");
    let g1 = flag(ctx.core, owner, "u05_sub_main_g1");
    let main_hs = ctx
        .core
        .vm_aux
        .piles
        .remove(&(owner, "u05_sub_main_hs".to_string()))
        .unwrap_or_default();
    let main_es = ctx
        .core
        .vm_aux
        .piles
        .remove(&(owner, "u05_sub_main_es".to_string()))
        .unwrap_or_default();
    {
        let p = ctx.core.player_mut(owner);
        p.max_hp = main_max_hp;
        p.hp = main_hp;
        p.hujia = main_hujia;
        p.general_slots = [
            (g0 > 0).then(|| (g0 - 1) as u16),
            (g1 > 0).then(|| (g1 - 1) as u16),
        ];
        p.hand = main_hs;
        p.equips = main_es;
    }

    let sk_cnt = flag(ctx.core, owner, "u05_sub_main_sk_cnt");
    for i in 0..sk_cnt {
        let encoded = flag(ctx.core, owner, &format!("u05_sub_main_sk:{i}"));
        if encoded & (1u32 << 31) != 0 {
            let def = (encoded & 0xffff) as u16;
            let inst = SkillInstance {
                owner,
                enabled: (encoded >> 16) & 1 != 0,
                once_used: (encoded >> 17) & 1 != 0,
                expires_end_of_turn: (encoded >> 18) & 1 != 0,
                rounds: ((encoded >> 19) & 0xff) as u8,
            };
            ctx.core.skills.insert((owner, def), inst);
        }
        set_flag(ctx.core, owner, &format!("u05_sub_main_sk:{i}"), 0);
    }
    for key in [
        "u05_sub_main_hp",
        "u05_sub_main_maxhp",
        "u05_sub_main_hujia",
        "u05_sub_main_g0",
        "u05_sub_main_g1",
        "u05_sub_main_sk_cnt",
    ] {
        set_flag(ctx.core, owner, key, 0);
    }
}
