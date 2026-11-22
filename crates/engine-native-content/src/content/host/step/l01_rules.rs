//! L01 `AI优化` 核心规则、以逸待劳、模式修正、伪禁与昭告执行机。

use super::*;
use crate::catalog::Catalog;
use crate::choice::ChoiceOption;
use crate::ids::SkillDefId;
use crate::snap::{ask, ExpectedAsk};
use crate::state::{GameMode, GameStateCore, Role, SkillInstance};
use crate::vocabulary::host::{
    L01_FAKE_PROHIBITED_KEY, L01_FIND_ZHONG_KEY, L01_FIX_CF_DELTA_PROMPT,
    L01_FIX_CF_SKILL_PROMPT, L01_FIX_ECF_KEY, L01_FIX_EQZ_KEY, L01_FIX_QZ_DELTA_PROMPT,
    L01_FIX_QZ_TARGET_PROMPT, L01_NEIKEY_KEY, L01_NEIKEY_RECOVER_PROMPT,
};

fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux.flags.get(&(p, key.to_string())).copied().unwrap_or(0)
}

fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

fn l01_rule_enabled(
    core: &GameStateCore,
    catalog: &Catalog,
    cfg_flag: &str,
    skill_key: &str,
) -> bool {
    if flag(core, 0, cfg_flag) > 0 {
        return true;
    }
    catalog.skill_by_name(skill_key).is_some_and(|def| {
        (0..core.players.len() as PlayerId).any(|p| core.has_skill(p, def))
    })
}

fn lord_seat_of(core: &GameStateCore) -> Option<PlayerId> {
    match &core.mode {
        GameMode::Identity { roles } => {
            roles.iter().position(|r| *r == Role::Lord).map(|i| i as PlayerId)
        }
        GameMode::Doudizhu { lord } => Some(*lord),
        _ => None,
    }
}

fn role_ban_prefix_for_seat(core: &GameStateCore, seat: PlayerId) -> Option<&'static str> {
    match &core.mode {
        GameMode::Identity { roles } => match roles.get(seat as usize) {
            Some(Role::Lord) => Some("l01_ban:zhu"),
            Some(Role::Loyalist) => Some("l01_ban:zhong"),
            Some(Role::Rebel) => Some("l01_ban:fan"),
            Some(Role::Traitor) => Some("l01_ban:nei"),
            None => None,
        },
        GameMode::Doudizhu { lord } => {
            if seat == *lord {
                Some("l01_ban:dizhu")
            } else {
                Some("l01_ban:nongmin")
            }
        }
        _ => None,
    }
}

fn is_general_banned_for_seat(core: &GameStateCore, seat: PlayerId, gen_name: &str) -> bool {
    if flag(core, 0, &format!("l01_ban:all:{gen_name}")) > 0 {
        return true;
    }
    role_ban_prefix_for_seat(core, seat)
        .is_some_and(|prefix| flag(core, 0, &format!("{prefix}:{gen_name}")) > 0)
}

fn apply_general_swap_for_seat(
    core: &mut GameStateCore,
    catalog: &Catalog,
    seat: PlayerId,
    slot: usize,
    new_gen: u16,
    new_max_hp: u32,
) {
    let old_gen = core.player(seat).general_slots[slot];
    let old_max_hp = old_gen
        .and_then(|g| catalog.general_meta(g))
        .map(|m| m.3)
        .unwrap_or(core.player(seat).max_hp);
    core.player_mut(seat).general_slots[slot] = Some(new_gen);
    let diff = new_max_hp as i32 - old_max_hp as i32;
    let next_max = (core.player(seat).max_hp as i32 + diff).max(1) as u32;
    core.player_mut(seat).max_hp = next_max;
    core.player_mut(seat).hp = (core.player(seat).hp + diff).clamp(1, next_max as i32);

    if let Some(old) = old_gen {
        if let Some((_, _, old_sk, _, _)) = catalog.general_meta(old) {
            for sk in old_sk {
                if let Some(def) = catalog.skill_by_name(sk) {
                    if !catalog.extension_global_skills.contains(&def) {
                        core.skills.remove(&(seat, def));
                    }
                }
            }
        }
    }
    if let Some((_, _, new_sk, _, _)) = catalog.general_meta(new_gen) {
        for sk in new_sk {
            if let Some(def) = catalog.skill_by_name(sk) {
                core.skills.insert(
                    (seat, def),
                    SkillInstance {
                        owner: seat,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }
    }
    if matches!(core.mode, GameMode::Kingdoms)
        && slot == 0
        && matches!(
            core.player(seat).kingdom_identity,
            crate::state::KingdomIdentity::Declared(_)
        )
    {
        if let Some((_, faction, _, _, _)) = catalog.general_meta(new_gen) {
            core.player_mut(seat).kingdom_identity =
                crate::state::KingdomIdentity::Declared(
                    GameStateCore::faction_name(faction).to_string(),
                );
        }
    }
}

/// L01 开局规则初始化（`_aiyh_firstKs` `fixFour` / `_findZhong` / `_aiyh_fake_prohibited` `Wj`）。
pub fn l01_on_game_start(core: &mut GameStateCore, catalog: &Catalog) {
    // 1. `fixFour` (`AI优化/extension.js:67-77`): 4 人身份局改 1 主(+1血) 1 内 2 明反。
    if flag(core, 0, "l01_cfg:fix_four") > 0 && flag(core, 0, "l01_fix_four_applied") == 0 {
        let mut lords = Vec::new();
        let mut shown_rebels = Vec::new();
        if let GameMode::Identity { roles } = &mut core.mode {
            if roles.len() == 4 {
                for (seat, r) in roles.iter_mut().enumerate() {
                    let s = seat as PlayerId;
                    if *r == Role::Lord {
                        lords.push(s);
                    } else if *r == Role::Loyalist {
                        *r = Role::Rebel;
                        shown_rebels.push(s);
                    }
                }
            }
        }
        if !shown_rebels.is_empty() {
            for s in lords {
                core.player_mut(s).max_hp = core.player(s).max_hp.saturating_add(1);
                core.player_mut(s).hp = core.player(s).hp.saturating_add(1);
            }
            for s in shown_rebels {
                set_flag(core, s, "l01_role_shown", 1);
            }
        }
        set_flag(core, 0, "l01_fix_four_applied", 1);
    }

    // 2. `_findZhong` (`AI优化/extension.js:675-703`): 主公开局仅己侧知晓 1 名忠臣。
    if l01_rule_enabled(core, catalog, "l01_cfg:find_zhong", L01_FIND_ZHONG_KEY)
        && flag(core, 0, "l01_find_zhong_done") == 0
    {
        if let GameMode::Identity { roles } = &core.mode {
            if let Some(lord) = roles.iter().position(|r| *r == Role::Lord).map(|i| i as PlayerId) {
                let loyalists: Vec<PlayerId> = roles
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, r)| {
                        (*r == Role::Loyalist && core.player(idx as PlayerId).alive)
                            .then_some(idx as PlayerId)
                    })
                    .collect();
                if !loyalists.is_empty() {
                    let pick_idx = core.rng.gen_range(loyalists.len() as u32) as usize;
                    let picked = loyalists[pick_idx];
                    set_flag(core, lord, "l01_find_zhong_target_plus1", u32::from(picked) + 1);
                    set_flag(core, 0, "l01_find_zhong_done", 1);
                }
            }
        }
    }

    // 3. `_aiyh_fake_prohibited` (`AI优化/extension.js:439-557`): 伪禁替换 AI 所选禁将。
    if l01_rule_enabled(core, catalog, "l01_cfg:fake_prohibited", L01_FAKE_PROHIBITED_KEY)
        && flag(core, 0, "l01_wj_pass_done") == 0
    {
        let human_seat = flag(core, 0, "l01_human_seat") as PlayerId;
        let max_cands = match flag(core, 0, "l01_cfg:wjs") {
            n @ 1..=10 => n as usize,
            _ => 6,
        };
        let total_gens = catalog.general_count() as u16;
        let mut used: Vec<u16> = core
            .players
            .iter()
            .flat_map(|p| p.general_slots.iter().flatten().copied())
            .collect();
        for seat in 0..core.players.len() as PlayerId {
            if seat == human_seat && flag(core, 0, "l01_apply_wj_seat0") == 0 {
                continue;
            }
            for slot in 0..2usize {
                let Some(gen_id) = core.player(seat).general_slots[slot] else {
                    continue;
                };
                let Some((gname, _, _, _, _)) = catalog.general_meta(gen_id) else {
                    continue;
                };
                if !is_general_banned_for_seat(core, seat, gname) {
                    continue;
                }
                let mut pool: Vec<(u16, u32)> = Vec::new();
                for cand in 0..total_gens {
                    if used.contains(&cand) {
                        continue;
                    }
                    let Some((cname, _, _, max_hp, _)) = catalog.general_meta(cand) else {
                        continue;
                    };
                    if is_general_banned_for_seat(core, seat, cname) {
                        continue;
                    }
                    pool.push((cand, max_hp));
                }
                core.rng.shuffle(&mut pool);
                pool.truncate(max_cands);
                let picked = if pool.is_empty() {
                    None
                } else {
                    let idx = core.rng.gen_range(pool.len() as u32) as usize;
                    Some(pool[idx])
                };
                if let Some((replacement, new_mhp)) = picked {
                    used.retain(|&g| g != gen_id);
                    used.push(replacement);
                    apply_general_swap_for_seat(core, catalog, seat, slot, replacement, new_mhp);
                    set_flag(
                        core,
                        seat,
                        &format!("l01_wj_replaced:{slot}"),
                        u32::from(replacement) + 1,
                    );
                } else {
                    set_flag(core, 0, "l01_wj_empty_pool", 1);
                }
            }
        }
        set_flag(core, 0, "l01_wj_pass_done", 1);
    }

    if catalog
        .skill_by_name(crate::vocabulary::host::L01_FIRST_KS_KEY)
        .is_some()
    {
        set_flag(core, 0, "l01_first_ks_done", 1);
    }
}

pub(crate) fn activate_l01_skill(
    skill_name: &str,
    behavior: HostBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
        || ctx.state.turn.current != owner
    {
        return None;
    }
    match behavior {
        HostBehavior::L01NeiKey => {
            let GameMode::Identity { roles } = &ctx.state.mode else {
                return None;
            };
            if roles.get(owner as usize) != Some(&Role::Traitor) {
                return None;
            }
            if flag(ctx.state, owner, "l01_neikey_used") > 0 {
                return None;
            }
            let mode = match ctx
                .state
                .vm_aux
                .flags
                .get(&(0, "l01_cfg:neikey_mode".to_string()))
                .copied()
            {
                Some(1) => 1,
                Some(2) => 2,
                _ => 0,
            };
            if mode == 0 {
                return None;
            }
            if mode == 1 && flag(ctx.state, owner, "l01_role_shown") > 0 {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: L01_NEIKEY_KEY.to_string(),
                behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        HostBehavior::L01AiRuleHooks if skill_name == L01_FIX_EQZ_KEY => {
            if owner != flag(ctx.state, 0, "l01_human_seat") as PlayerId
                || flag(ctx.state, 0, "l01_cfg:fix_eqz") == 0
            {
                return None;
            }
            if !ctx
                .state
                .alive_players()
                .any(|p| ctx.state.player(p).general_slots[0].is_some())
            {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill_name.to_string(),
                behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        HostBehavior::L01AiRuleHooks if skill_name == L01_FIX_ECF_KEY => {
            if owner != flag(ctx.state, 0, "l01_human_seat") as PlayerId
                || flag(ctx.state, 0, "l01_cfg:fix_ecf") == 0
                || ctx.state.skills.is_empty()
            {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill_name.to_string(),
                behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        _ => None,
    }
}

pub(super) fn is_primary_alive_holder_or_dying(
    core: &GameStateCore,
    catalog: &Catalog,
    skill_name: &str,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let Some(def) = catalog.skill_by_name(skill_name) else {
        return false;
    };
    if let Some(first) = core.alive_players().find(|&p| core.has_skill(p, def)) {
        first == owner
    } else {
        matches!(event, EventData::Death { player, .. } if *player == owner)
            && core.has_skill(owner, def)
    }
}

pub(super) fn migrate_l01_trackers(core: &mut GameStateCore, catalog: &Catalog, dead: PlayerId) {
    let Some(next) = core.alive_players().find(|&p| p != dead) else {
        return;
    };
    for name in [
        crate::vocabulary::host::L01_FIRST_KS_KEY,
        L01_FIND_ZHONG_KEY,
        L01_FAKE_PROHIBITED_KEY,
        crate::content::host::L01_ATTZHI_KEY,
        "attcard",
        "attdamage",
        "attlosehp",
        "attrecover",
        "_yingshilanggu",
    ] {
        let Some(def) = catalog.skill_by_name(name) else {
            continue;
        };
        if core.has_skill(dead, def)
            && !core.alive_players().any(|p| p != dead && core.has_skill(p, def))
        {
            core.skills.insert(
                (next, def),
                SkillInstance {
                    owner: next,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
    }
}

pub(crate) fn filter_l01_first_ks(
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !is_primary_alive_holder_or_dying(
        ctx.state,
        ctx.catalog,
        crate::vocabulary::host::L01_FIRST_KS_KEY,
        owner,
        event,
    ) {
        return false;
    }
    match event {
        EventData::Phase { begin: true, .. } => {
            flag(ctx.state, 0, "l01_first_ks_done") == 0
                || (flag(ctx.state, 0, "l01_cfg:fix_four") > 0
                    && flag(ctx.state, 0, "l01_fix_four_applied") == 0)
                || (flag(ctx.state, 0, "l01_cfg:find_zhong") > 0
                    && flag(ctx.state, 0, "l01_find_zhong_done") == 0)
        }
        EventData::Death { .. } => true,
        _ => false,
    }
}

pub(crate) fn machine_l01(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    if skill.behavior == HostBehavior::L01AttitudeTracker {
        return super::l01_attitude::machine_l01_att(skill, owner, event);
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: match event {
            EventData::Death { player, .. } => Some(*player),
            _ => None,
        },
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(crate) fn step_l01_rule(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::L01FirstKs => {
            if let Some(dying) = m.target {
                super::l01_attitude::update_public_endgame_state(ctx.core, Some(dying));
                migrate_l01_trackers(ctx.core, ctx.catalog, dying);
            } else {
                l01_on_game_start(ctx.core, ctx.catalog);
            }
            Flow::Done
        }
        HostBehavior::L01YiyiCard => super::l01_yiyi::step_l01_yiyi(m, ctx),
        HostBehavior::L01NeiKey => step_l01_neikey(m, ctx),
        HostBehavior::L01AiRuleHooks => step_l01_runtime_tuning(m, ctx),
        HostBehavior::L01AttitudeTracker => super::l01_attitude::step_l01_att(m, ctx),
        _ => Flow::Done,
    }
}

fn step_l01_neikey(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l01_neikey_used", 1);
            set_flag(ctx.core, owner, "l01_role_shown", 1);
            set_flag(ctx.core, owner, "l01_gjcx_expose", 0);
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            m.stage = HostStage::SelectNature;
            Flow::WaitChoice {
                observer: owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: L01_NEIKEY_RECOVER_PROMPT,
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            }
        }
        HostStage::SelectNature => {
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                ctx.recover(owner, 1, None);
                if let Some(lord) = lord_seat_of(ctx.core).filter(|&l| ctx.core.player(l).alive) {
                    ctx.recover(lord, 1, None);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_l01_runtime_tuning(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.def_key == L01_FIX_EQZ_KEY {
        match m.stage {
            HostStage::Act => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|&p| ctx.core.player(p).general_slots[0].is_some())
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: L01_FIX_QZ_TARGET_PROMPT,
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                m.stage = HostStage::SelectNature;
                Flow::WaitChoice {
                    observer: owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: L01_FIX_QZ_DELTA_PROMPT,
                        options: (0..=3).map(ChoiceOption::Tag).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectNature => {
                if let (Some(t), Some(ChoiceOption::Tag(tag))) =
                    (m.target, ctx.answer().first().copied())
                {
                    if let Some(g) = ctx.core.player(t).general_slots[0] {
                        if let Some((gname, _, _, _, _)) = ctx.catalog.general_meta(g) {
                            let delta: i32 = match tag {
                                0 => 10,
                                1 => 50,
                                2 => -10,
                                _ => -50,
                            };
                            let key = format!("l01_rt_qz_pct:{gname}");
                            let cur = ctx.core.vm_aux.storage.get(&key).copied().unwrap_or(100);
                            let next = (cur as i32 + delta).clamp(5, 10_000) as u32;
                            ctx.core.vm_aux.storage.insert(key, next);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else if m.def_key == L01_FIX_ECF_KEY {
        match m.stage {
            HostStage::Act => {
                let options: Vec<ChoiceOption> = l01_fix_cf_skill_options(ctx.core);
                if options.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: L01_FIX_CF_SKILL_PROMPT,
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Skill(def)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.cards.clear();
                ctx.core
                    .vm_aux
                    .storage
                    .insert("l01_pending_cf_skill".to_string(), u32::from(def));
                m.stage = HostStage::SelectNature;
                Flow::WaitChoice {
                    observer: owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: L01_FIX_CF_DELTA_PROMPT,
                        options: (0..=3).map(ChoiceOption::Tag).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectNature => {
                let pending = ctx.core.vm_aux.storage.remove("l01_pending_cf_skill");
                if let (Some(def_u32), Some(ChoiceOption::Tag(tag))) =
                    (pending, ctx.answer().first().copied())
                {
                    if let Some(sk) = ctx.catalog.skills.get(def_u32 as usize) {
                        let delta: i32 = match tag {
                            0 => 10,
                            1 => 50,
                            2 => -10,
                            _ => -50,
                        };
                        let key = format!("l01_rt_cf_pct:{}", sk.name);
                        let cur = ctx.core.vm_aux.storage.get(&key).copied().unwrap_or(100);
                        let next = (cur as i32 + delta).clamp(1, 10_000) as u32;
                        ctx.core.vm_aux.storage.insert(key, next);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        Flow::Done
    }
}

fn l01_fix_cf_skill_options(core: &GameStateCore) -> Vec<ChoiceOption> {
    let mut defs: Vec<SkillDefId> = core.skills.keys().map(|(_, d)| *d).collect();
    defs.sort_unstable();
    defs.dedup();
    defs.into_iter().map(ChoiceOption::Skill).collect()
}

pub(crate) fn active_ask_l01(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
        HostBehavior::L01YiyiCard => {
            super::l01_yiyi::active_ask_l01_yiyi(stage, owner, target, core, catalog)
        }
        HostBehavior::L01NeiKey if stage == HostStage::SelectNature => Some(ask(
            owner,
            L01_NEIKEY_RECOVER_PROMPT,
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        )),
        HostBehavior::L01AiRuleHooks => match stage {
            HostStage::TargetChoice => Some(ask(
                owner,
                L01_FIX_QZ_TARGET_PROMPT,
                ChoiceKind::SelectPlayers,
                core.alive_players()
                    .filter(|&p| core.player(p).general_slots[0].is_some())
                    .map(ChoiceOption::Player)
                    .collect(),
                1,
                1,
                true,
            )),
            HostStage::SelectCard => Some(ask(
                owner,
                L01_FIX_CF_SKILL_PROMPT,
                ChoiceKind::PickTag,
                l01_fix_cf_skill_options(core),
                1,
                1,
                true,
            )),
            HostStage::SelectNature => Some(ask(
                owner,
                if target.is_some() {
                    L01_FIX_QZ_DELTA_PROMPT
                } else {
                    L01_FIX_CF_DELTA_PROMPT
                },
                ChoiceKind::PickTag,
                (0..=3).map(ChoiceOption::Tag).collect(),
                1,
                1,
                true,
            )),
            _ => None,
        },
        _ => None,
    }
}
