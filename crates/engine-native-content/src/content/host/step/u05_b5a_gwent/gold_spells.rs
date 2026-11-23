//! `card/gwent/card.js` 12 张金卡法术结算。

use super::super::super::*;
use super::{
    clean_gwent_name, enemies_of, finish_gwent_card, pull_card_matching, GOLD_SPELL_PICK_NAMES,
};
use crate::state::{GameMode, GameStateCore};

pub(super) fn step_gold_spell(m: &mut HostMachine, ctx: &mut Ctx<'_>, cname: &str) -> Flow {
    match cname {
        "gw_hudiewu" => step_hudiewu(m, ctx),
        "gw_yigeniyin" => step_yigeniyin(m, ctx),
        "gw_leizhoushu" => step_leizhoushu(m, ctx),
        "gw_aerdeyin" => step_aerdeyin(m, ctx),
        "gw_ansha" => step_ansha(m, ctx),
        "gw_xinsheng" => step_xinsheng(m, ctx),
        "gw_niuquzhijing" => step_niuquzhijing(m, ctx),
        "gw_zhongmozhizhan" => step_zhongmozhizhan(m, ctx),
        "gw_ganhan" => step_ganhan(m, ctx),
        "gw_huangjiashenpan" => step_huangjiashenpan(m, ctx),
        "gw_tunshi" => step_tunshi(m, ctx),
        "gw_chongci" => step_chongci(m, ctx),
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 蝴蝶舞 (`card/gwent/card.js:100`): 所有其他角色装备区/判定区每张牌替换为
/// `gw_dieyi_equip1..5` / `gw_dieyi_judge`（若对应唯一牌不在牌堆/弃堆则用假标锁定）。
fn step_hudiewu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let targets: Vec<PlayerId> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|&p| {
            p != m.owner
                && ctx.core.player(p).alive
                && (!ctx.core.player(p).equips.is_empty() || !ctx.core.player(p).judge.is_empty())
        })
        .collect();
    for target in targets {
        let equips = std::mem::take(&mut ctx.core.player_mut(target).equips);
        let mut kept_equips = Vec::new();
        for c in equips {
            let slot_tag = ctx
                .catalog
                .card_slot_of(ctx.core, c)
                .map(|s| match s {
                    crate::catalog::EquipSlot::Weapon => "gw_dieyi_equip1",
                    crate::catalog::EquipSlot::Armor => "gw_dieyi_equip2",
                    crate::catalog::EquipSlot::HorsePlus => "gw_dieyi_equip3",
                    crate::catalog::EquipSlot::HorseMinus => "gw_dieyi_equip4",
                    crate::catalog::EquipSlot::Treasure => "gw_dieyi_equip5",
                })
                .unwrap_or("gw_dieyi_equip1");
            let cur_name = ctx.catalog.cards[c as usize].name;
            if cur_name == slot_tag {
                kept_equips.push(c);
                continue;
            }
            if let Some(dieyi_c) = pull_card_matching(ctx, |id| ctx.catalog.cards[id as usize].name == slot_tag) {
                let equip_skills = ctx.catalog.cards[c as usize].equip_skills.clone();
                for sk in equip_skills {
                    if let Some(def) = ctx.catalog.skill_by_name(sk) {
                        let still = kept_equips
                            .iter()
                            .any(|kc| ctx.catalog.cards[*kc as usize].equip_skills.contains(&sk));
                        if !still {
                            ctx.core.skills.remove(&(target, def));
                        }
                    }
                }
                ctx.core.declared_slots.remove(&c);
                ctx.to_discard(c);
                ctx.note_cards_lost(target, &[c]);
                ctx.emit(EventData::CardsLost {
                    player: target,
                    cards: vec![c],
                    origins: vec![crate::event::LostOrigin::Equip],
                });
                kept_equips.push(dieyi_c);
            } else {
                // 单张卡池下的保底假标：禁用原装备技能、保留关键快照一致性
                for sk in &ctx.catalog.cards[c as usize].equip_skills {
                    if let Some(def) = ctx.catalog.skill_by_name(sk) {
                        if let Some(inst) = ctx.core.skills.get_mut(&(target, def)) {
                            inst.enabled = false;
                        }
                    }
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((target, format!("gw_dieyi_card:{c}")), 1);
                kept_equips.push(c);
            }
        }
        ctx.core.player_mut(target).equips = kept_equips;

        let judges = std::mem::take(&mut ctx.core.player_mut(target).judge);
        let mut kept_judges = Vec::new();
        for c in judges {
            if ctx.catalog.cards[c as usize].name == "gw_dieyi_judge" {
                kept_judges.push(c);
                continue;
            }
            if let Some(dieyi_j) =
                pull_card_matching(ctx, |id| ctx.catalog.cards[id as usize].name == "gw_dieyi_judge")
            {
                ctx.core.declared_slots.remove(&c);
                ctx.to_discard(c);
                ctx.note_cards_lost(target, &[c]);
                ctx.emit(EventData::CardsLost {
                    player: target,
                    cards: vec![c],
                    origins: vec![crate::event::LostOrigin::Judge],
                });
                kept_judges.push(dieyi_j);
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((target, format!("gw_dieyi_card:{c}")), 1);
                kept_judges.push(c);
            }
        }
        ctx.core.player_mut(target).judge = kept_judges;
    }
    finish_gwent_card(m, ctx, true)
}

/// 伊格尼印 (`card/gwent/card.js:158`): 先令一名体力最高敌方受 1 火伤，再令当前全部体力最高角色各受 1 火伤。
fn step_yigeniyin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let enemies = enemies_of(ctx.core, ctx.catalog, m.owner);
            let max_enemy_hp = enemies.iter().map(|&p| ctx.core.player(p).hp).max();
            if let Some(mhp) = max_enemy_hp {
                let top_enemies: Vec<PlayerId> = enemies
                    .into_iter()
                    .filter(|&p| ctx.core.player(p).hp == mhp)
                    .collect();
                if !top_enemies.is_empty() {
                    let pick =
                        top_enemies[ctx.core.rng.gen_range(top_enemies.len() as u32) as usize];
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(m.owner),
                        pick,
                        1,
                        crate::event::DamageNature::Fire,
                        m.card,
                    )));
                    m.stage = HostStage::TargetChoice;
                    return Flow::Yield;
                }
            }
            m.stage = HostStage::TargetChoice;
            step_yigeniyin(m, ctx)
        }
        HostStage::TargetChoice => {
            // 第一步火伤及死亡/触发全部结算完毕后，再重算当前全局最高体力并按座位序（逆序压栈）伤害
            if let Some(global_max) = ctx
                .core
                .alive_players()
                .map(|p| ctx.core.player(p).hp)
                .max()
            {
                let top_all: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(m.owner)
                    .into_iter()
                    .filter(|&p| ctx.core.player(p).alive && ctx.core.player(p).hp == global_max)
                    .collect();
                for t in top_all.into_iter().rev() {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(m.owner),
                        t,
                        1,
                        crate::event::DamageNature::Fire,
                        m.card,
                    )));
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 雷咒术 (`card/gwent/card.js:245`)：获得 `gw_leizhoushu`，已持有则重复次数 +1。
fn step_leizhoushu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(def) = ctx.catalog.skill_by_name("gw_leizhoushu") {
        let key = (m.owner, "gw_leizhoushu_count".to_string());
        if ctx.core.has_skill(m.owner, def) {
            let cur = ctx.core.vm_aux.flags.get(&key).copied().unwrap_or(1);
            ctx.core.vm_aux.flags.insert(key, cur.max(1) + 1);
        } else {
            ctx.insert_skill(
                (m.owner, def),
                crate::state::SkillInstance {
                    owner: m.owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
            ctx.core.vm_aux.flags.insert(key, 1);
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 阿尔德印 (`card/gwent/card.js:293`): 随机敌方受 1 伤，若正面朝上则翻面并获 `gw_aerdeyin`。
fn step_aerdeyin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let enemies = enemies_of(ctx.core, ctx.catalog, m.owner);
            if enemies.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            let target = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
            m.target = Some(target);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(m.owner),
                target,
                1,
                crate::event::DamageNature::Normal,
                m.card,
            )));
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(target) = m.target {
                if ctx.core.player(target).alive && !ctx.core.player(target).face_down {
                    ctx.turn_over(target);
                    if let Some(def) = ctx.catalog.skill_by_name("gw_aerdeyin") {
                        ctx.insert_skill(
                            (target, def),
                            crate::state::SkillInstance {
                                owner: target,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 暗杀 (`card/gwent/card.js:356`): 随机一名体力为 1 的敌方直接死亡。
fn step_ansha(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let candidates: Vec<PlayerId> = enemies_of(ctx.core, ctx.catalog, m.owner)
        .into_iter()
        .filter(|&p| ctx.core.player(p).hp == 1)
        .collect();
    if !candidates.is_empty() {
        let target = candidates[ctx.core.rng.gen_range(candidates.len() as u32) as usize];
        ctx.emit(EventData::Death {
            player: target,
            killer: None,
        });
    }
    finish_gwent_card(m, ctx, true)
}

/// 新生 (`card/gwent/card.js:398`): 从 12 名未登场武将中选 1，替换一名非暗置角色的武将并回复 1 体力。
fn step_xinsheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let used_gens: std::collections::BTreeSet<u16> = ctx
                .core
                .players
                .iter()
                .flat_map(|p| p.general_slots.iter().flatten().copied())
                .collect();
            let mut pool: Vec<u16> = (0..ctx.catalog.general_count() as u16)
                .filter(|g| !used_gens.contains(g) && ctx.catalog.general_meta(*g).is_some())
                .collect();
            if pool.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            let take = pool.len().min(12);
            let mut cands = Vec::with_capacity(take);
            for _ in 0..take {
                let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                cands.push(pool.swap_remove(idx));
            }
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "u05_gw_xinsheng_n".to_string()), cands.len() as u32);
            for (i, g) in cands.iter().enumerate() {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, format!("u05_gw_xinsheng_g:{i}")), u32::from(*g));
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gw_xinsheng_general",
                    options: (0..cands.len() as u8).map(ChoiceOption::Tag).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            let idx = match ctx.answer_first() {
                ChoiceOption::Tag(i) => i,
                _ => 0,
            };
            let Some(g_id) = ctx
                .core
                .vm_aux
                .flags
                .get(&(m.owner, format!("u05_gw_xinsheng_g:{idx}")))
                .copied()
            else {
                return finish_gwent_card(m, ctx, true);
            };
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "u05_gw_xinsheng_chosen".to_string()), g_id);
            let options: Vec<ChoiceOption> = ctx
                .core
                .seat_order_from(m.owner)
                .into_iter()
                .filter(|&p| {
                    ctx.core.player(p).alive && is_player_seen(ctx.core, p)
                })
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                clear_xinsheng_flags(ctx.core, m.owner);
                return finish_gwent_card(m, ctx, true);
            }
            m.stage = HostStage::TargetChoice;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gw_xinsheng_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::TargetChoice => {
            let chosen_gen = ctx
                .core
                .vm_aux
                .flags
                .get(&(m.owner, "u05_gw_xinsheng_chosen".to_string()))
                .copied()
                .map(|v| v as u16);
            clear_xinsheng_flags(ctx.core, m.owner);
            let target = ctx.answer().iter().find_map(|o| match *o {
                ChoiceOption::Player(p) => Some(p),
                _ => None,
            });
            if let (Some(target), Some(new_gen)) = (target, chosen_gen) {
                reinit_player_general(ctx, target, new_gen);
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

pub(super) fn is_player_seen(core: &GameStateCore, p: PlayerId) -> bool {
    if !matches!(core.mode, GameMode::Kingdoms) {
        return true;
    }
    core.player(p).revealed_generals[0] || core.player(p).revealed_generals[1]
}

fn clear_xinsheng_flags(core: &mut GameStateCore, owner: PlayerId) {
    core.vm_aux
        .flags
        .remove(&(owner, "u05_gw_xinsheng_n".to_string()));
    core.vm_aux
        .flags
        .remove(&(owner, "u05_gw_xinsheng_chosen".to_string()));
    for i in 0..12 {
        core.vm_aux
            .flags
            .remove(&(owner, format!("u05_gw_xinsheng_g:{i}")));
    }
}

fn reinit_player_general(ctx: &mut Ctx<'_>, target: PlayerId, new_gen: u16) {
    let old_hp = ctx.core.player(target).hp;
    let removed: Vec<crate::ids::SkillDefId> = ctx
        .core
        .player(target)
        .general_slots
        .iter()
        .flatten()
        .filter_map(|def| ctx.catalog.general_meta(*def))
        .flat_map(|meta| meta.2)
        .filter_map(|name| ctx.catalog.skill_by_name(name))
        .collect();
    for def in removed {
        let name = ctx.catalog.skills[def as usize].name;
        let still = ctx.catalog.extension_global_skills.contains(&def)
            || ctx
                .core
                .player(target)
                .equips
                .iter()
                .any(|card| ctx.catalog.cards[*card as usize].equip_skills.contains(&name));
        if !still {
            ctx.core.skills.remove(&(target, def));
        }
    }
    ctx.core.player_mut(target).general_slots = [Some(new_gen), None];
    ctx.core.player_mut(target).revealed_generals = [true, false];
    crate::state::apply_general_assembly(ctx.core, ctx.catalog, target, 0);
    let lord = ctx
        .core
        .identity_roles()
        .and_then(|roles| roles.get(target as usize))
        == Some(&crate::state::Role::Lord);
    if lord && ctx.core.player(target).max_hp != crate::state::INFINITE_HP {
        ctx.core.player_mut(target).max_hp += 1;
    }
    let max_hp = ctx.core.player(target).max_hp as i32;
    ctx.core.player_mut(target).hp = (old_hp + 1).min(max_hp);
}

/// 纽曲之镜 (`card/gwent/card.js:514`): 所有体力最多者体力与上限-1，最少者体力与上限+1（先快照判定）。
fn step_niuquzhijing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let alive: Vec<(PlayerId, i32)> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|&p| ctx.core.player(p).alive)
        .map(|p| (p, ctx.core.player(p).hp))
        .collect();
    if let (Some(max_hp), Some(min_hp)) = (
        alive.iter().map(|(_, h)| *h).max(),
        alive.iter().map(|(_, h)| *h).min(),
    ) {
        if max_hp != min_hp {
            for (p, h) in alive {
                if h == max_hp {
                    let pl = ctx.core.player_mut(p);
                    pl.hp -= 1;
                    pl.max_hp = pl.max_hp.saturating_sub(1);
                } else if h == min_hp {
                    let pl = ctx.core.player_mut(p);
                    pl.hp += 1;
                    if pl.max_hp != crate::state::INFINITE_HP {
                        pl.max_hp += 1;
                    }
                }
            }
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 终末之战 (`card/gwent/card.js:609`): 所有存活角色弃置区域内全部牌（无触发 `he`+`j`：`lose()._triggered = null`）。
fn step_zhongmozhizhan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let seats: Vec<PlayerId> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|&p| ctx.core.player(p).alive)
        .collect();
    for p in seats {
        let hand = std::mem::take(&mut ctx.core.player_mut(p).hand);
        let equips = std::mem::take(&mut ctx.core.player_mut(p).equips);
        let judge = std::mem::take(&mut ctx.core.player_mut(p).judge);
        for c in &equips {
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, *c);
        }
        let mut all = Vec::with_capacity(hand.len() + equips.len() + judge.len());
        all.extend(hand.iter().copied());
        all.extend(equips.iter().copied());
        all.extend(judge.iter().copied());
        for &c in &all {
            ctx.core.declared_slots.remove(&c);
            ctx.core
                .vm_aux
                .flags
                .remove(&(p, format!("gw_dieyi_card:{c}")));
            ctx.to_discard(c);
        }
        if !all.is_empty() {
            ctx.log(LogEvent::CardDiscarded {
                player: p,
                cards: all,
            });
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 干旱 (`card/gwent/card.js:661`): 所有角色依次失去 1 点体力上限。
fn step_ganhan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let seats: Vec<PlayerId> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|&p| ctx.core.player(p).alive)
        .collect();
    for p in seats {
        if ctx.core.player(p).alive {
            ctx.lose_max_hp(p, 1);
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 皇家审判 (`card/gwent/card.js:737`): 选择并获得另一张金卡法术。
fn step_huangjiashenpan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = GOLD_SPELL_PICK_NAMES
                .iter()
                .enumerate()
                .filter(|(_, name)| {
                    ctx.core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .any(|c| clean_gwent_name(ctx.catalog.cards[*c as usize].name) == **name)
                })
                .map(|(i, _)| ChoiceOption::Tag(i as u8))
                .collect();
            if opts.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gw_huangjiashenpan_pick",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            if let ChoiceOption::Tag(idx) = ctx.answer_first() {
                if let Some(&name) = GOLD_SPELL_PICK_NAMES.get(idx as usize) {
                    if let Some(c) = pull_card_matching(ctx, |id| {
                        clean_gwent_name(ctx.catalog.cards[id as usize].name) == name
                    }) {
                        ctx.core.player_mut(m.owner).hand.push(c);
                        ctx.emit(EventData::CardsDrawn {
                            player: m.owner,
                            cards: vec![c],
                        });
                    }
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 吞噬 (`card/gwent/card.js:779`): 偷取随机一名敌方的一项技能，自减 1 体力/上限，目标加 1 上限/回 1 体力。
fn step_tunshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let candidates: Vec<(PlayerId, Vec<crate::ids::SkillDefId>)> =
                enemies_of(ctx.core, ctx.catalog, m.owner)
                    .into_iter()
                    .filter_map(|t| {
                        let skills: Vec<_> = ctx
                            .core
                            .skills
                            .iter()
                            .filter(|((p, _), _)| *p == t)
                            .map(|((_, def), _)| *def)
                            .filter(|&def| {
                                let sname = ctx.catalog.skills[def as usize].name;
                                let is_equip = ctx
                                    .core
                                    .player(t)
                                    .equips
                                    .iter()
                                    .any(|ec| ctx.catalog.cards[*ec as usize].equip_skills.contains(&sname));
                                !is_equip && !ctx.catalog.is_charlotte_skill(def)
                            })
                            .collect();
                        (!skills.is_empty()).then_some((t, skills))
                    })
                    .collect();
            if candidates.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            let pick_t = ctx.core.rng.gen_range(candidates.len() as u32) as usize;
            let (target, skills) = &candidates[pick_t];
            let target = *target;
            let pick_s = ctx.core.rng.gen_range(skills.len() as u32) as usize;
            let sk_def = skills[pick_s];
            ctx.core.skills.remove(&(target, sk_def));
            ctx.insert_skill(
                (m.owner, sk_def),
                crate::state::SkillInstance {
                    owner: m.owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
            m.target = Some(target);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::LoseHp {
                player: m.owner,
                amount: 1,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(m.owner).alive {
                ctx.lose_max_hp(m.owner, 1);
            }
            if let Some(target) = m.target {
                if ctx.core.player(target).alive {
                    if ctx.core.player(target).max_hp != crate::state::INFINITE_HP {
                        ctx.core.player_mut(target).max_hp += 1;
                    }
                    ctx.recover(target, 1, None);
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

fn chongci_encode_slot(slot: Option<crate::catalog::EquipSlot>) -> u32 {
    match slot {
        Some(crate::catalog::EquipSlot::Weapon) => 1,
        Some(crate::catalog::EquipSlot::Armor) => 2,
        Some(crate::catalog::EquipSlot::HorsePlus) => 3,
        Some(crate::catalog::EquipSlot::HorseMinus) => 4,
        Some(crate::catalog::EquipSlot::Treasure) => 5,
        None => 0,
    }
}

fn chongci_decode_slot(code: u32) -> Option<crate::catalog::EquipSlot> {
    match code {
        1 => Some(crate::catalog::EquipSlot::Weapon),
        2 => Some(crate::catalog::EquipSlot::Armor),
        3 => Some(crate::catalog::EquipSlot::HorsePlus),
        4 => Some(crate::catalog::EquipSlot::HorseMinus),
        5 => Some(crate::catalog::EquipSlot::Treasure),
        _ => None,
    }
}

/// 冲刺 (`card/gwent/card.js:844`): 弃置全部 hand+equips，换取同类别牌各 1 张 + 1 张银/铜法术，随后 `tempHide()`。
fn step_chongci(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let hs = std::mem::take(&mut ctx.core.player_mut(owner).hand);
            let es = std::mem::take(&mut ctx.core.player_mut(owner).equips);
            ctx.core
                .vm_aux
                .flags
                .insert((owner, "u05_gw_cc_hs_n".to_string()), hs.len() as u32);
            for (i, &c) in hs.iter().enumerate() {
                let k = ctx.catalog.card_kind(c);
                let n = clean_gwent_name(ctx.catalog.cards[c as usize].name);
                let bits = u32::from(k.is_basic_kind())
                    | (u32::from(k.equip_slot().is_some()) << 1)
                    | (u32::from(is_gwent_spell_name(n)) << 2);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("u05_gw_cc_h:{i}")), (c << 4) | bits);
            }
            ctx.core
                .vm_aux
                .flags
                .insert((owner, "u05_gw_cc_es_n".to_string()), es.len() as u32);
            for (i, &c) in es.iter().enumerate() {
                let code = chongci_encode_slot(ctx.catalog.card_slot_of(ctx.core, c));
                ctx.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("u05_gw_cc_e:{i}")), (c << 4) | code);
            }
            for &c in &es {
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
            }
            for &c in hs.iter().chain(es.iter()) {
                ctx.core.declared_slots.remove(&c);
                ctx.to_discard(c);
            }
            if !hs.is_empty() {
                ctx.note_cards_lost(owner, &hs);
                ctx.emit(EventData::CardsLost {
                    player: owner,
                    cards: hs.clone(),
                    origins: vec![crate::event::LostOrigin::Hand; hs.len()],
                });
            }
            if !es.is_empty() {
                ctx.note_cards_lost(owner, &es);
                ctx.emit(EventData::CardsLost {
                    player: owner,
                    cards: es.clone(),
                    origins: vec![crate::event::LostOrigin::Equip; es.len()],
                });
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let hs_n = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "u05_gw_cc_hs_n".to_string()))
                .unwrap_or(0);
            let mut hs_slots = Vec::with_capacity(hs_n as usize);
            for i in 0..hs_n {
                if let Some(v) = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("u05_gw_cc_h:{i}")))
                {
                    let orig_cid = v >> 4;
                    let orig_name = clean_gwent_name(ctx.catalog.cards[orig_cid as usize].name);
                    let is_basic = (v & 1) != 0;
                    let is_equip = (v & 2) != 0;
                    let is_spell = (v & 4) != 0;
                    hs_slots.push((orig_name, is_basic, is_equip, is_spell));
                }
            }
            let es_n = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "u05_gw_cc_es_n".to_string()))
                .unwrap_or(0);
            let mut es_slots = Vec::with_capacity(es_n as usize);
            for i in 0..es_n {
                if let Some(v) = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("u05_gw_cc_e:{i}")))
                {
                    let orig_cid = v >> 4;
                    let orig_name = clean_gwent_name(ctx.catalog.cards[orig_cid as usize].name);
                    let slot = chongci_decode_slot(v & 0xF);
                    es_slots.push((orig_name, slot));
                }
            }
            if !ctx.core.player(owner).alive {
                return finish_gwent_card(m, ctx, true);
            }
            let mut gained = Vec::new();
            for (orig_name, is_basic, is_equip, is_spell) in hs_slots {
                let found = pull_card_matching(ctx, |cid| {
                    let n = clean_gwent_name(ctx.catalog.cards[cid as usize].name);
                    let k = ctx.catalog.card_kind(cid);
                    n != orig_name
                        && ((is_spell && is_gwent_spell_name(n))
                            || (!is_spell
                                && k.is_basic_kind() == is_basic
                                && k.equip_slot().is_some() == is_equip
                                && !is_gwent_spell_name(n)))
                })
                .or_else(|| {
                    pull_card_matching(ctx, |cid| {
                        let k = ctx.catalog.card_kind(cid);
                        let n = clean_gwent_name(ctx.catalog.cards[cid as usize].name);
                        (is_spell && is_gwent_spell_name(n))
                            || (!is_spell
                                && k.is_basic_kind() == is_basic
                                && k.equip_slot().is_some() == is_equip)
                    })
                });
                if let Some(c) = found {
                    gained.push(c);
                }
            }
            if let Some(non_gold) = pull_card_matching(ctx, |cid| {
                let n = clean_gwent_name(ctx.catalog.cards[cid as usize].name);
                super::SILVER_SPELL_PICK_NAMES.contains(&n)
                    || super::BRONZE_SPELL_NAMES.iter().any(|&(bn, _)| bn == n)
            }) {
                gained.push(non_gold);
            }
            if !gained.is_empty() && ctx.core.player(owner).alive {
                ctx.core.player_mut(owner).hand.extend(gained.iter().copied());
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: gained,
                });
            }
            for (orig_name, slot) in es_slots {
                if let Some(slot) = slot {
                    let pick_id = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .find(|&cid| {
                            clean_gwent_name(ctx.catalog.cards[cid as usize].name) != orig_name
                                && ctx.catalog.card_slot_of(ctx.core, cid) == Some(slot)
                        })
                        .or_else(|| {
                            ctx.core
                                .deck
                                .iter()
                                .chain(ctx.core.discard.iter())
                                .copied()
                                .find(|&cid| ctx.catalog.card_slot_of(ctx.core, cid) == Some(slot))
                        });
                    let eq =
                        pick_id.and_then(|target_id| pull_card_matching(ctx, |cid| cid == target_id));
                    if let Some(eq_id) = eq {
                        let replaced = ctx
                            .core
                            .player(owner)
                            .equips
                            .iter()
                            .copied()
                            .find(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(slot));
                        if let Some(old) = replaced {
                            ctx.core.player_mut(owner).equips.retain(|c| *c != old);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, old);
                            ctx.core.declared_slots.remove(&old);
                            ctx.core
                                .vm_aux
                                .flags
                                .remove(&(owner, format!("gw_dieyi_card:{old}")));
                            ctx.to_discard(old);
                        }
                        ctx.core.player_mut(owner).equips.push(eq_id);
                        crate::kernel::grant_equip_skills(ctx, owner, eq_id);
                    }
                }
            }
            if ctx.core.player(owner).alive {
                ctx.core.player_mut(owner).yinshen_active = true;
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

fn is_gwent_spell_name(name: &str) -> bool {
    let name = clean_gwent_name(name);
    GOLD_SPELL_PICK_NAMES.contains(&name)
        || name == "gw_huangjiashenpan"
        || super::SILVER_SPELL_PICK_NAMES.contains(&name)
        || super::BRONZE_SPELL_NAMES.iter().any(|&(bn, _)| bn == name)
}
