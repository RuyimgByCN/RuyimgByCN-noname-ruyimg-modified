//! `character/gwent/card.js` 前半段：`gw_xianzumaijiu` 与前 9 张冒险金卡。

use super::super::super::*;
use super::{clean_gwent_name, enemies_of, finish_gwent_card, friends_of, skills};
use crate::catalog::Catalog;
use crate::state::GameStateCore;

pub(super) fn step_adventure_a(m: &mut HostMachine, ctx: &mut Ctx<'_>, cname: &str) -> Flow {
    match cname {
        "gw_xianzumaijiu" => step_xianzumaijiu(m, ctx),
        "gwmaoxian_yioufeisi" => step_yioufeisi(m, ctx),
        "gwmaoxian_luoqi" => step_luoqi(m, ctx),
        "gwmaoxian_jieluote" => step_jieluote(m, ctx),
        "gwmaoxian_yenaifa" => step_yenaifa(m, ctx),
        "gwmaoxian_telisi" => step_telisi(m, ctx),
        "gwmaoxian_hengsaite" => step_hengsaite(m, ctx),
        "gwmaoxian_fuertaisite" => step_fuertaisite(m, ctx),
        "gwmaoxian_laduoweide" => step_laduoweide(m, ctx),
        "gwmaoxian_enxier" => step_enxier(m, ctx),
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 先祖麦酒 (`character/gwent/card.js:9`): 回合出牌阶段付与 `gw_xianzumaijiu` 临时技直至本回合/杀结算后。
fn step_xianzumaijiu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    ctx.core
        .vm_aux
        .flags
        .insert((m.owner, "gw_xianzumaijiu_used_play".to_string()), 1);
    if let Some(def) = ctx.catalog.skill_by_name("gw_xianzumaijiu") {
        ctx.add_skill_until_turn_end(m.owner, def);
    }
    finish_gwent_card(m, ctx, false)
}

/// 伊欧菲斯 (`character/gwent/card.js:119`): 两名其他角色互相视为使用一张【杀】。
fn step_yioufeisi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if m.targets.len() < 2 {
                return finish_gwent_card(m, ctx, true);
            }
            let (t0, t1) = (m.targets[0], m.targets[1]);
            m.stage = HostStage::DrawSettle;
            if ctx.core.player(t0).alive && ctx.core.player(t1).alive {
                ctx.use_virtual_card(t0, CardKind::Sha, vec![t1]);
                return Flow::Yield;
            }
            finish_gwent_card(m, ctx, true)
        }
        HostStage::DrawSettle => {
            if m.targets.len() >= 2 {
                let (t0, t1) = (m.targets[0], m.targets[1]);
                if ctx.core.player(t0).alive && ctx.core.player(t1).alive {
                    m.stage = HostStage::SelectNature;
                    ctx.use_virtual_card(t1, CardKind::Sha, vec![t0]);
                    return Flow::Yield;
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 罗契 (`character/gwent/card.js:158`): 无视距离对目标使用【杀】，其余角色可依次对目标使用【杀】。
fn step_luoqi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Some(victim) = m.target else {
                return finish_gwent_card(m, ctx, true);
            };
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "u05_gw_luoqi_victim".to_string()), u32::from(victim) + 1);
            let others: Vec<PlayerId> = ctx
                .core
                .seat_order_from(m.owner)
                .into_iter()
                .filter(|&p| p != m.owner && p != victim && ctx.core.player(p).alive)
                .collect();
            m.targets = others;
            m.stage = HostStage::ShuchuiTargetLoop;
            ctx.use_virtual_card(m.owner, CardKind::Sha, vec![victim]);
            Flow::Yield
        }
        HostStage::ShuchuiTargetLoop => {
            let victim = ctx
                .core
                .vm_aux
                .flags
                .get(&(m.owner, "u05_gw_luoqi_victim".to_string()))
                .copied()
                .and_then(|v| v.checked_sub(1))
                .map(|v| v as PlayerId);
            let Some(_victim) = victim.filter(|&v| ctx.core.player(v).alive) else {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(m.owner, "u05_gw_luoqi_victim".to_string()));
                return finish_gwent_card(m, ctx, true);
            };
            while !m.targets.is_empty() {
                let responder = m.targets.remove(0);
                if !ctx.core.player(responder).alive {
                    continue;
                }
                let shas = luoqi_sha_options(ctx.core, ctx.catalog, responder);
                if shas.is_empty() {
                    continue;
                }
                m.target = Some(responder);
                m.stage = HostStage::SelectCard;
                return Flow::WaitChoice {
                    observer: responder,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "gwmaoxian_luoqi_sha",
                        options: shas,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(m.owner, "u05_gw_luoqi_victim".to_string()));
            finish_gwent_card(m, ctx, true)
        }
        HostStage::SelectCard => {
            let victim = ctx
                .core
                .vm_aux
                .flags
                .get(&(m.owner, "u05_gw_luoqi_victim".to_string()))
                .copied()
                .and_then(|v| v.checked_sub(1))
                .map(|v| v as PlayerId);
            if let (Some(responder), Some(victim), Some(ChoiceOption::Card(sha))) =
                (m.target, victim, ctx.answer().first().copied())
            {
                if ctx.core.player(responder).hand.contains(&sha) && ctx.core.player(victim).alive {
                    let is_wuyao =
                        clean_gwent_name(ctx.catalog.cards[sha as usize].name) == "gw_wuyao";
                    let use_as = is_wuyao.then_some(CardKind::Sha);
                    m.stage = HostStage::ShuchuiTargetLoop;
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        sha,
                        responder,
                        vec![victim],
                        use_as,
                    )));
                    return Flow::Yield;
                }
            }
            m.stage = HostStage::ShuchuiTargetLoop;
            step_luoqi(m, ctx)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

pub(super) fn luoqi_sha_options(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
) -> Vec<ChoiceOption> {
    core.player(responder)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            catalog.card_kind(c).is_sha_like()
                || clean_gwent_name(catalog.cards[c as usize].name) == "gw_wuyao"
        })
        .map(ChoiceOption::Card)
        .collect()
}

/// 杰洛特 (`character/gwent/card.js:201`): 若目标体力全场最高且 > 2 则造成 2 点伤害，否则 1 点。
fn step_jieluote(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(t) = m.target {
        if ctx.core.player(t).alive {
            let max_hp = ctx
                .core
                .alive_players()
                .map(|p| ctx.core.player(p).hp)
                .max()
                .unwrap_or(0);
            let thp = ctx.core.player(t).hp;
            let dmg = if thp == max_hp && thp > 2 { 2 } else { 1 };
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(m.owner),
                t,
                dmg,
                crate::event::DamageNature::Normal,
                m.card,
            )));
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 叶奈法 (`character/gwent/card.js:261`): 令至多 3 名随机敌方各受 1 项减益。
fn step_yenaifa(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let mut enemies = enemies_of(ctx.core, ctx.catalog, m.owner);
    let take = enemies.len().min(3);
    let mut picked = Vec::with_capacity(take);
    for _ in 0..take {
        let idx = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
        picked.push(enemies.swap_remove(idx));
    }
    let ordered: Vec<PlayerId> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|p| picked.contains(p))
        .collect();
    for t in ordered {
        if ctx.core.player(t).alive {
            skills::apply_debuff(ctx, t);
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 特莉斯 (`character/gwent/card.js:294`): 令至多 3 名随机友方及自己各获 1 项增益。
fn step_telisi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let mut friends = friends_of(ctx.core, ctx.catalog, m.owner);
    let take = friends.len().min(3);
    let mut picked = Vec::with_capacity(take + 1);
    for _ in 0..take {
        let idx = ctx.core.rng.gen_range(friends.len() as u32) as usize;
        picked.push(friends.swap_remove(idx));
    }
    if !picked.contains(&m.owner) {
        picked.push(m.owner);
    }
    let ordered: Vec<PlayerId> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|p| picked.contains(p))
        .collect();
    for t in ordered {
        if ctx.core.player(t).alive {
            skills::apply_buff(ctx, t);
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 亨赛特 (`character/gwent/card.js:329`): 视为使用【万箭齐发】，每造成伤害令你获得 1 张【杀】。
fn step_hengsaite(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let targets: Vec<PlayerId> = ctx
                .core
                .seat_order_from(m.owner)
                .into_iter()
                .filter(|&p| p != m.owner && ctx.core.player(p).alive)
                .collect();
            if targets.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            if let Some(def) = ctx.catalog.skill_by_name("gwmaoxian_hengsaite_sha") {
                ctx.add_skill_until_turn_end(m.owner, def);
            }
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "gwmaoxian_hengsaite_active".to_string()), 1);
            m.stage = HostStage::DrawSettle;
            ctx.use_virtual_card(m.owner, CardKind::ArcheryAttack, targets);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            ctx.core
                .vm_aux
                .flags
                .remove(&(m.owner, "gwmaoxian_hengsaite_active".to_string()));
            if let Some(def) = ctx.catalog.skill_by_name("gwmaoxian_hengsaite_sha") {
                ctx.core.skills.remove(&(m.owner, def));
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 弗尔泰斯特 (`character/gwent/card.js:371`): 1~2 名角色获得 1 点护甲。
fn step_fuertaisite(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    for &t in &m.targets {
        if ctx.core.player(t).alive {
            ctx.change_hujia(t, 1);
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 拉多维德 (`character/gwent/card.js:406`): 封印目标非锁定技直至其下个阶段/回合结束，然后造成 1 伤。
fn step_laduoweide(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(t) = m.target {
        if ctx.core.player(t).alive {
            skills::apply_fengyin_to_target(ctx, t);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(m.owner),
                t,
                1,
                crate::event::DamageNature::Normal,
                m.card,
            )));
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 恩希尔 (`character/gwent/card.js:438`): 与手牌数相差 <= 1 的目标交换全部手牌。
fn step_enxier(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(t) = m.target {
        if t != m.owner && ctx.core.player(t).alive && ctx.core.player(m.owner).alive {
            let h_own = std::mem::take(&mut ctx.core.player_mut(m.owner).hand);
            let h_tgt = std::mem::take(&mut ctx.core.player_mut(t).hand);
            ctx.core.player_mut(m.owner).hand = h_tgt.clone();
            ctx.core.player_mut(t).hand = h_own.clone();
            if !h_own.is_empty() {
                ctx.note_cards_lost(m.owner, &h_own);
                ctx.emit(EventData::CardsMoved {
                    cards: h_own,
                    from: m.owner,
                    to: t,
                });
            }
            if !h_tgt.is_empty() {
                ctx.note_cards_lost(t, &h_tgt);
                ctx.emit(EventData::CardsMoved {
                    cards: h_tgt,
                    from: t,
                    to: m.owner,
                });
            }
        }
    }
    finish_gwent_card(m, ctx, true)
}
