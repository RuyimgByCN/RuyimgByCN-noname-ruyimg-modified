//! L19 末日乱斗（`末日乱斗/extension.js`）共享规则助手与内核生命周期钩子。

use crate::catalog::Catalog;
use crate::ids::PlayerId;
use crate::machine::Ctx;
use crate::state::{GameMode, GameStateCore, KingdomIdentity, Role};

fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

fn has_skill_named(core: &GameStateCore, catalog: &Catalog, p: PlayerId, name: &str) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.has_skill(p, def))
}

fn player_role(core: &GameStateCore, p: PlayerId) -> Option<Role> {
    core.identity_roles().and_then(|r| r.get(p as usize).copied())
}

/// `chanceIdentity0..15`（`末日乱斗/extension.js:843-1032`）：返回 `(slot_idx, kingdom_label, len_gate, next_skill)`。
pub(in crate::content::host) fn chance_identity_step(
    name: &str,
) -> Option<(u32, &'static str, Option<usize>, Option<&'static str>)> {
    match name {
        "chanceIdentity0" => Some((0, "yi", None, Some("chanceIdentity"))),
        "chanceIdentity" => Some((1, "er", Some(2), Some("chanceIdentity2"))),
        "chanceIdentity2" => Some((2, "san", Some(3), Some("chanceIdentity3"))),
        "chanceIdentity3" => Some((3, "si", Some(4), Some("chanceIdentity4"))),
        "chanceIdentity4" => Some((4, "wu1", Some(5), Some("chanceIdentity5"))),
        "chanceIdentity5" => Some((5, "liu", Some(6), Some("chanceIdentity6"))),
        "chanceIdentity6" => Some((6, "qi", Some(7), Some("chanceIdentity7"))),
        "chanceIdentity7" => Some((7, "ba", Some(8), Some("chanceIdentity8"))),
        "chanceIdentity8" => Some((8, "jiu1", Some(9), Some("chanceIdentity9"))),
        "chanceIdentity9" => Some((9, "shi", Some(10), Some("chanceIdentity10"))),
        "chanceIdentity10" => Some((10, "shiyi", Some(11), Some("chanceIdentity11"))),
        "chanceIdentity11" => Some((11, "shier", Some(12), Some("chanceIdentity12"))),
        "chanceIdentity12" => Some((12, "shisan", Some(13), Some("chanceIdentity13"))),
        "chanceIdentity13" => Some((13, "shisi", Some(14), Some("chanceIdentity14"))),
        "chanceIdentity14" => Some((14, "shiwu", Some(15), Some("chanceIdentity15"))),
        "chanceIdentity15" => Some((15, "shiliu", Some(16), None)),
        _ => None,
    }
}

/// 执行 `chanceIdentity` 链：自 `start_owner` 起沿存活座环依次将下家的 `kingdom_identity`
/// 置为 `Declared(label)`，并严格按 `alive_players > gate` 继续触发下一座。
pub(in crate::content::host) fn execute_chance_identity(
    core: &mut GameStateCore,
    start_owner: PlayerId,
    skill_name: &str,
) {
    let alive_count = core.alive_players().count();
    let mut cur_seat = start_owner;
    let mut cur_name = skill_name;
    for _ in 0..18 {
        let Some((idx, label, gate, next_skill)) = chance_identity_step(cur_name) else {
            break;
        };
        let next = core.next_alive(cur_seat);
        let declared = KingdomIdentity::Declared(label.to_string());
        if matches!(core.mode, GameMode::Kingdoms) {
            let _ = core.set_kingdom_identity(next, declared);
        } else {
            core.player_mut(next).kingdom_identity = declared;
        }
        set_flag(core, next, "l19_chance_identity_slot", idx + 1);
        set_flag(core, cur_seat, "l19_chance_identity_assigned", idx + 1);
        match (gate, next_skill) {
            (None, Some(ns)) => {
                cur_seat = next;
                cur_name = ns;
            }
            (Some(g), Some(ns)) if alive_count > g => {
                cur_seat = next;
                cur_name = ns;
            }
            _ => break,
        }
    }
}

/// 收集无尽模式 `wujinmoshiRE2` 可从 `victim` 身上选学的候选技能列表（`末日乱斗/extension.js:1799,1881`）。
pub(in crate::content::host) fn collect_wujin_stealable_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    receiver: PlayerId,
    victim: PlayerId,
) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    let mut push_skill = |raw: &'static str| {
        if raw.starts_with('_')
            || raw.starts_with("wujinmoshi")
            || out.contains(&raw)
            || catalog
                .skill_by_name(raw)
                .is_some_and(|d| core.has_skill(receiver, d))
        {
            return;
        }
        if catalog.skill_by_name(raw).is_some() {
            out.push(raw);
        }
    };
    for g_opt in core.player(victim).general_slots {
        if let Some(g_idx) = g_opt {
            if let Some((_, _, sks, _, _)) = catalog.general_meta(g_idx) {
                for sk in sks {
                    if let Some(d) = catalog.skill_by_name(sk) {
                        push_skill(catalog.skills[d as usize].name);
                    }
                }
            }
        }
    }
    for (&(p, def), inst) in &core.skills {
        if p == victim && inst.enabled {
            if let Some(sk) = catalog.skills.get(def as usize) {
                push_skill(sk.name);
            }
        }
    }
    out
}

/// `phaseUseBegin` L19 钩子：递增出牌阶段序号以区分同回合多出牌阶段（`_huzhu` `usable: 1`）。
pub(crate) fn play_begin_l19(ctx: &mut Ctx<'_>, player: PlayerId) {
    if ctx.catalog.skill_by_name("_huzhu").is_none() {
        return;
    }
    let next_seq = flag(ctx.core, player, "_huzhu_play_seq").saturating_add(1);
    set_flag(ctx.core, player, "_huzhu_play_seq", next_seq);
}

/// 仅当存在存活 `Role::Lord` 持有 `_relifeskill` 时，死者技能表保留供复活后继续生效。
pub(crate) fn host_preserves_skills_on_death(
    core: &GameStateCore,
    catalog: &Catalog,
    _player: PlayerId,
) -> bool {
    if catalog.skill_by_name("_relifeskill").is_none() {
        return false;
    }
    (0..core.players.len() as PlayerId).any(|p| {
        core.player(p).alive
            && player_role(core, p) == Some(Role::Lord)
            && has_skill_named(core, catalog, p, "_relifeskill")
    })
}

/// 轮次开始钩子（`末日乱斗/extension.js:2329-2374` `_relifeskill`）：
/// 仅当场上存在存活 `Role::Lord` 且持有 `_relifeskill` 时，对全体阵亡角色倒数 `zuan_relife`；
/// 归零按原 `max_hp` 复活、摸 `ceil(level * 0.5)`（逆序压栈保持按座序先摸），并重设 `relife = ceil(level * 0.4 + 0.1)` 与护盾。
pub(crate) fn host_on_round_start(ctx: &mut Ctx<'_>) {
    if !host_preserves_skills_on_death(ctx.core, ctx.catalog, 0) {
        return;
    }
    let mut revived_draws: Vec<(PlayerId, u32)> = Vec::new();
    for dd in 0..ctx.core.players.len() as PlayerId {
        if ctx.core.player(dd).alive || ctx.core.player(dd).max_hp == 0 {
            continue;
        }
        let next_relife = flag(ctx.core, dd, "zuan_relife") as i32 - 1;
        if next_relife >= 1 {
            set_flag(ctx.core, dd, "zuan_relife", next_relife as u32);
            continue;
        }
        let orig_max = ctx.core.player(dd).max_hp.max(1);
        {
            let pl = ctx.core.player_mut(dd);
            pl.alive = true;
            pl.dying = false;
            pl.hp = orig_max as i32;
        }
        let level = flag(ctx.core, dd, "zuan_level")
            .max(flag(ctx.core, dd, "_levelskill"))
            .max(1);
        let draw_n = (level + 1) / 2;
        if draw_n > 0 {
            revived_draws.push((dd, draw_n));
        }
        let reset_relife = (level * 4 + 1 + 9) / 10;
        set_flag(ctx.core, dd, "zuan_relife", reset_relife);
        if has_skill_named(ctx.core, ctx.catalog, dd, "zuan_hudun") {
            let max_hj = flag(ctx.core, dd, "zuan_maxhujia");
            set_flag(ctx.core, dd, "zuan_hudun", max_hj);
        }
        let cnt = flag(ctx.core, dd, "_relifeskill_revived") + 1;
        set_flag(ctx.core, dd, "_relifeskill_revived", cnt);
    }
    for (dd, draw_n) in revived_draws.into_iter().rev() {
        ctx.start_draw(dd, draw_n);
    }
}

/// 回复前置钩子（`末日乱斗/extension.js:1994-2017` `_wujinmoshiRecover` / `_wujinmoshiRecoverE`）：
/// 仅修改当前回复事件 `amount`（不额外发回复、不于 `Phase::End` 凭空自愈）。
pub(crate) fn host_on_recover_before(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    amount: &mut u32,
) {
    if *amount == 0 {
        return;
    }
    if ctx.catalog.skill_by_name("_wujinmoshiRecover").is_none()
        && ctx.catalog.skill_by_name("_wujinmoshiRecoverE").is_none()
    {
        return;
    }
    if player == 0 {
        if has_skill_named(ctx.core, ctx.catalog, 0, "_wujinmoshiRecover") {
            let bonus =
                flag(ctx.core, 0, "wujin_recover_bonus").max(flag(ctx.core, 0, "l19_wujin_recover_bonus"));
            if bonus > 0 {
                *amount = amount.saturating_add(bonus);
                let prev = flag(ctx.core, 0, "_wujinmoshiRecover_healed");
                set_flag(ctx.core, 0, "_wujinmoshiRecover_healed", prev.saturating_add(bonus));
            }
        }
        return;
    }
    if has_skill_named(ctx.core, ctx.catalog, player, "_wujinmoshiRecoverE")
        || has_skill_named(ctx.core, ctx.catalog, 0, "_wujinmoshiRecoverE")
    {
        let dxb = flag(ctx.core, 0, "wujinmoshiDXB").max(flag(ctx.core, player, "wujinmoshiDXB"));
        let sub =
            flag(ctx.core, 0, "wujinmoshiRecoverI").max(flag(ctx.core, player, "wujinmoshiRecoverI"));
        let delta = (dxb / 10) as i32 - sub as i32;
        if delta > 0 {
            *amount = amount.saturating_add(delta as u32);
        } else if delta < 0 {
            *amount = amount.saturating_sub((-delta) as u32);
        }
    }
}
