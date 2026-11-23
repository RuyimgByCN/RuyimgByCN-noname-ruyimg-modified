//! `character/gwent/card.js` 后半段：后 11 张冒险金卡。

use super::super::super::*;
use super::{
    clean_gwent_name, discard_he_card, finish_gwent_card, pull_card_matching, GOLD_SPELL_PICK_NAMES,
    SILVER_SPELL_PICK_NAMES,
};
use crate::catalog::Catalog;
use crate::state::GameStateCore;

pub(super) fn step_adventure_b(m: &mut HostMachine, ctx: &mut Ctx<'_>, cname: &str) -> Flow {
    match cname {
        "gwmaoxian_fulisi" => step_fulisi(m, ctx),
        "gwmaoxian_kaerweite" => step_kaerweite(m, ctx),
        "gwmaoxian_bulanwang" => step_bulanwang(m, ctx),
        "gwmaoxian_kuite" => step_kuite(m, ctx),
        "gwmaoxian_haluo" => step_haluo(m, ctx),
        "gwmaoxian_dagong" => step_dagong(m, ctx),
        "gwmaoxian_gaier" => step_gaier(m, ctx),
        "gwmaoxian_airuiting" => step_airuiting(m, ctx),
        "gwmaoxian_aisinie" => step_aisinie(m, ctx),
        "gwmaoxian_falanxisika" => step_falanxisika(m, ctx),
        "gwmaoxian_huoge" => super::adventure_huoge::step_huoge(m, ctx),
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 符里斯 (`character/gwent/card.js:474`): 选 1~3 名其他角色，弃置其共计至多两张手牌。
fn step_fulisi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let opts = fulisi_card_options(ctx.core, &m.targets);
            if opts.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            let max = opts.len().min(2);
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gwmaoxian_fulisi_discard",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            }
        }
        HostStage::SelectCard => {
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .take(2)
                .collect();
            for c in picks {
                let holder = m
                    .targets
                    .iter()
                    .copied()
                    .find(|&p| ctx.core.player(p).alive && ctx.core.player(p).hand.contains(&c));
                if let Some(t) = holder {
                    ctx.discard_from_hand(t, c);
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

pub(super) fn fulisi_card_options(core: &GameStateCore, targets: &[PlayerId]) -> Vec<ChoiceOption> {
    targets
        .iter()
        .copied()
        .filter(|&p| core.player(p).alive)
        .flat_map(|p| core.player(p).hand.iter().copied())
        .map(ChoiceOption::Card)
        .collect()
}

/// 卡尔维特 (`character/gwent/card.js:526`): 依次获得 1~2 名目标各一张手牌。
fn step_kaerweite(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage == HostStage::Act {
        let ordered: Vec<PlayerId> = ctx
            .core
            .seat_order_from(m.owner)
            .into_iter()
            .filter(|p| m.targets.contains(p))
            .collect();
        m.targets = ordered;
        m.stage = HostStage::TargetChoice;
    }
    if m.stage == HostStage::SelectCard {
        if let (Some(t), Some(ChoiceOption::Card(picked))) = (m.target, ctx.answer().first().copied()) {
            if ctx.core.player(t).hand.contains(&picked) {
                ctx.take_hand_card(t, picked);
                ctx.note_cards_lost(t, &[picked]);
                ctx.core.player_mut(m.owner).hand.push(picked);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![picked],
                    from: t,
                    to: m.owner,
                });
            }
        }
        m.stage = HostStage::TargetChoice;
    }
    while !m.targets.is_empty() {
        let next = m.targets.remove(0);
        if !ctx.core.player(next).alive || ctx.core.player(next).hand.is_empty() {
            continue;
        }
        m.target = Some(next);
        m.stage = HostStage::SelectCard;
        return Flow::WaitChoice {
            observer: m.owner,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "gwmaoxian_kaerweite_pick",
                options: ctx
                    .core
                    .player(next)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect(),
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        };
    }
    finish_gwent_card(m, ctx, true)
}

/// 布兰王 (`character/gwent/card.js:561`): 可弃至多两张牌摸两倍牌，然后跳过弃牌阶段。
fn step_bulanwang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .chain(ctx.core.player(m.owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                ctx.core.player_mut(m.owner).skip_discard_phase = true;
                return finish_gwent_card(m, ctx, true);
            }
            let max = opts.len().min(2);
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gwmaoxian_bulanwang_discard",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            }
        }
        HostStage::SelectCard => {
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .take(2)
                .collect();
            let count = picks.len() as u32;
            for c in picks {
                discard_he_card(ctx, m.owner, c);
            }
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "u05_gw_bulan_n".to_string()), count);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let count = ctx
                .core
                .vm_aux
                .flags
                .remove(&(m.owner, "u05_gw_bulan_n".to_string()))
                .unwrap_or(0);
            if count > 0 && ctx.core.player(m.owner).alive {
                ctx.start_draw(m.owner, count * 2);
            }
            ctx.core.player_mut(m.owner).skip_discard_phase = true;
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 奎特 (`character/gwent/card.js:603`): 视为对目标连续使用两张【决斗】。
fn step_kuite(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Some(t) = m.target else {
                return finish_gwent_card(m, ctx, true);
            };
            m.stage = HostStage::DrawSettle;
            if ctx.core.player(m.owner).alive && ctx.core.player(t).alive {
                ctx.use_virtual_card(m.owner, CardKind::Duel, vec![t]);
                return Flow::Yield;
            }
            finish_gwent_card(m, ctx, true)
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target {
                if ctx.core.player(m.owner).alive && ctx.core.player(t).alive {
                    m.stage = HostStage::SelectNature;
                    ctx.use_virtual_card(m.owner, CardKind::Duel, vec![t]);
                    return Flow::Yield;
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 哈洛 (`character/gwent/card.js:637`): 对全体体力最少角色造成 1 点伤害。
fn step_haluo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(min_hp) = ctx
        .core
        .alive_players()
        .map(|p| ctx.core.player(p).hp)
        .min()
    {
        let targets: Vec<PlayerId> = ctx
            .core
            .seat_order_from(m.owner)
            .into_iter()
            .filter(|&p| ctx.core.player(p).alive && ctx.core.player(p).hp == min_hp)
            .collect();
        for t in targets.into_iter().rev() {
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

/// 达贡 (`character/gwent/card.js:668`): 令目标及相邻角色同时获得刺骨寒霜/碧日浓雾/倾盆大雨。
fn step_dagong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(t) = m.target {
        let mut group = vec![t];
        let (prev, next) = ctx.core.alive_neighbors(t);
        for p in [prev, next].into_iter().flatten() {
            if !group.contains(&p) {
                group.push(p);
            }
        }
        let pairs = [
            (GW_CIGU_STATE_KEY, "gw_ciguhanshuang"),
            (GW_BIRIN_STATE_KEY, "gw_birinongwu"),
            (GW_QINPEN_STATE_KEY, "gw_qinpendayu"),
        ];
        for g in group {
            if !ctx.core.player(g).alive {
                continue;
            }
            for (primary, alt) in pairs {
                let def = ctx
                    .catalog
                    .skill_by_name(primary)
                    .or_else(|| ctx.catalog.skill_by_name(alt));
                let alt_def = ctx.catalog.skill_by_name(alt);
                if let Some(def) = def {
                    if !ctx.core.skills.contains_key(&(g, def))
                        && !alt_def.is_some_and(|ad| ctx.core.skills.contains_key(&(g, ad)))
                    {
                        ctx.insert_skill(
                            (g, def),
                            crate::state::SkillInstance {
                                owner: g,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                }
            }
        }
    }
    finish_gwent_card(m, ctx, true)
}

/// 盖尔 (`character/gwent/card.js:704`): 令目标加 1 点体力并加 1 点上限，或失 1 点体力并失 1 点上限。
fn step_gaier(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if m.target.is_none() {
                return finish_gwent_card(m, ctx, true);
            }
            m.stage = HostStage::SelectNature;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gwmaoxian_gaier_branch",
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectNature => {
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    match ctx.answer_first() {
                        ChoiceOption::Tag(0) => {
                            if ctx.core.player(t).max_hp != crate::state::INFINITE_HP {
                                ctx.core.player_mut(t).max_hp += 1;
                            }
                            ctx.recover(t, 1, None);
                        }
                        _ => {
                            m.stage = HostStage::TargetChoice;
                            ctx.emit(EventData::LoseHp {
                                player: t,
                                amount: 1,
                            });
                            return Flow::Yield;
                        }
                    }
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        HostStage::TargetChoice => {
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    ctx.lose_max_hp(t, 1);
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 艾瑞汀 (`character/gwent/card.js:751`): 所有其他角色选择使用【杀】或失去 1 点体力。
fn step_airuiting(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage == HostStage::Act && m.targets.is_empty() && m.target.is_none() {
        m.targets = ctx
            .core
            .seat_order_from(m.owner)
            .into_iter()
            .filter(|&p| p != m.owner && ctx.core.player(p).alive)
            .collect();
    }
    match m.stage {
        HostStage::SelectCard => {
            let Some(p) = m.target else {
                return finish_gwent_card(m, ctx, true);
            };
            if let Some(ChoiceOption::Card(sha)) = ctx.answer().first().copied() {
                let targets = airuiting_sha_targets_for_card(ctx.core, ctx.catalog, p, Some(sha));
                if ctx.core.player(p).hand.contains(&sha) && !targets.is_empty() {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((m.owner, "u05_gw_air_sha".to_string()), sha + 1);
                    m.stage = HostStage::TargetChoice;
                    return Flow::WaitChoice {
                        observer: p,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "gwmaoxian_airuiting_target",
                            options: targets,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
            }
            m.stage = HostStage::Act;
            if ctx.core.player(p).alive {
                ctx.emit(EventData::LoseHp {
                    player: p,
                    amount: 1,
                });
                return Flow::Yield;
            }
        }
        HostStage::TargetChoice => {
            let Some(p) = m.target else {
                return finish_gwent_card(m, ctx, true);
            };
            let sha = ctx
                .core
                .vm_aux
                .flags
                .remove(&(m.owner, "u05_gw_air_sha".to_string()))
                .and_then(|v| v.checked_sub(1));
            m.stage = HostStage::Act;
            if let (Some(sha), Some(ChoiceOption::Player(vt))) = (sha, ctx.answer().first().copied()) {
                if ctx.core.player(p).hand.contains(&sha) && ctx.core.player(vt).alive {
                    let is_wuyao =
                        clean_gwent_name(ctx.catalog.cards[sha as usize].name) == "gw_wuyao";
                    let use_as = is_wuyao.then_some(CardKind::Sha);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        sha,
                        p,
                        vec![vt],
                        use_as,
                    )));
                    return Flow::Yield;
                }
            }
            if ctx.core.player(p).alive {
                ctx.emit(EventData::LoseHp {
                    player: p,
                    amount: 1,
                });
                return Flow::Yield;
            }
        }
        _ => {}
    }

    while !m.targets.is_empty() {
        let p = m.targets.remove(0);
        if !ctx.core.player(p).alive {
            continue;
        }
        m.target = Some(p);
        let shas = airuiting_sha_options(ctx.core, ctx.catalog, p);
        if !shas.is_empty() && !crate::kernel::sha_blocked_for(ctx.core, ctx.catalog, p) {
            m.stage = HostStage::SelectCard;
            return Flow::WaitChoice {
                observer: p,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gwmaoxian_airuiting_sha",
                    options: shas,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        ctx.emit(EventData::LoseHp {
            player: p,
            amount: 1,
        });
        return Flow::Yield;
    }
    finish_gwent_card(m, ctx, true)
}

pub(super) fn airuiting_sha_options(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> Vec<ChoiceOption> {
    super::adventure_a::luoqi_sha_options(core, catalog, p)
        .into_iter()
        .filter(|opt| {
            let ChoiceOption::Card(sha) = *opt else {
                return false;
            };
            !airuiting_sha_targets_for_card(core, catalog, p, Some(sha)).is_empty()
        })
        .collect()
}

pub(super) fn airuiting_sha_targets_for_card(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    card: Option<CardId>,
) -> Vec<ChoiceOption> {
    let (eff_kind, nature, is_black, range_free) = if let Some(c) = card {
        let is_wuyao = clean_gwent_name(catalog.cards[c as usize].name) == "gw_wuyao";
        let kind = if is_wuyao {
            CardKind::Sha
        } else {
            catalog.card_kind(c)
        };
        let nature = match kind {
            CardKind::FireSha => crate::event::DamageNature::Fire,
            CardKind::LeiSha => crate::event::DamageNature::Thunder,
            _ => crate::event::DamageNature::Normal,
        };
        if crate::rvm_bridge::modifiers::eval_card_use_canceled(core, catalog, p, c, kind, nature) {
            return Vec::new();
        }
        let black = matches!(
            catalog.effective_card_suit(core, p, c),
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        );
        let unlimited = crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
            core, catalog, p, c, kind, nature,
        );
        (kind, nature, black, unlimited)
    } else {
        if crate::kernel::sha_blocked_for(core, catalog, p) {
            return Vec::new();
        }
        (CardKind::Sha, crate::event::DamageNature::Normal, false, false)
    };
    let molicha_def = catalog.skill_by_name(crate::vocabulary::host::GJ_FOOD_MOLICHA_KEY);
    let remen_def = (eff_kind == CardKind::Sha)
        .then(|| catalog.skill_by_name(crate::vocabulary::names::BOSS_REMEN))
        .flatten();
    let range = core.attack_range(catalog, p);
    core.seat_order_from(p)
        .into_iter()
        .filter(|&t| {
            if crate::kernel::sha_user_or_target_blocked(core, catalog, p, t) {
                return false;
            }
            if !range_free
                && core.computed_distance(catalog, p, t) > range
                && core.turn.zhufangshenshi_target != Some(t)
            {
                return false;
            }
            if remen_def.is_some_and(|d| {
                core.has_skill(t, d)
                    && !core.player(t).equips.iter().any(|c| {
                        catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Armor)
                    })
            }) {
                return false;
            }
            if let Some(c) = card {
                if is_black && molicha_def.is_some_and(|d| core.skills.contains_key(&(t, d))) {
                    return false;
                }
                if crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core, catalog, p, t, c, eff_kind, nature,
                ) {
                    return false;
                }
            }
            true
        })
        .map(ChoiceOption::Player)
        .collect()
}

/// 埃丝涅 (`character/gwent/card.js:796`): 回复 1 点体力并选择获得一张银卡法术。
fn step_aisinie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if ctx.core.player(m.owner).alive {
                ctx.recover(m.owner, 1, None);
            }
            let opts: Vec<ChoiceOption> = SILVER_SPELL_PICK_NAMES
                .iter()
                .enumerate()
                .filter(|(_, n)| {
                    ctx.core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .any(|c| clean_gwent_name(ctx.catalog.cards[*c as usize].name) == **n)
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
                    prompt: "gwmaoxian_aisinie_pick",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            if let ChoiceOption::Tag(idx) = ctx.answer_first() {
                if let Some(&name) = SILVER_SPELL_PICK_NAMES.get(idx as usize) {
                    if let Some(c) =
                        pull_card_matching(ctx, |id| {
                            clean_gwent_name(ctx.catalog.cards[id as usize].name) == name
                        })
                    {
                        if ctx.core.player(m.owner).alive {
                            ctx.core.player_mut(m.owner).hand.push(c);
                            ctx.emit(EventData::CardsDrawn {
                                player: m.owner,
                                cards: vec![c],
                            });
                        } else {
                            ctx.to_discard(c);
                        }
                    }
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

/// 法兰茜斯卡 (`character/gwent/card.js:833`): 随机展示 3 张金卡法术并立即使用其中一张。
fn step_falanxisika(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let mut avail: Vec<usize> = GOLD_SPELL_PICK_NAMES
                .iter()
                .enumerate()
                .filter(|(_, n)| {
                    ctx.core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .any(|c| clean_gwent_name(ctx.catalog.cards[*c as usize].name) == **n)
                })
                .map(|(i, _)| i)
                .collect();
            if avail.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            let take = avail.len().min(3);
            let mut picks = Vec::with_capacity(take);
            for _ in 0..take {
                let r = ctx.core.rng.gen_range(avail.len() as u32) as usize;
                picks.push(avail.swap_remove(r) as u32);
            }
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "u05_gw_falan_n".to_string()), picks.len() as u32);
            for (i, &g_idx) in picks.iter().enumerate() {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, format!("u05_gw_falan_g:{i}")), g_idx);
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gwmaoxian_falanxisika_pick",
                    options: picks.into_iter().map(|t| ChoiceOption::Tag(t as u8)).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            for i in 0..3 {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(m.owner, format!("u05_gw_falan_g:{i}")));
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(m.owner, "u05_gw_falan_n".to_string()));
            if let ChoiceOption::Tag(idx) = ctx.answer_first() {
                if let Some(&name) = GOLD_SPELL_PICK_NAMES.get(idx as usize) {
                    if let Some(gcard) =
                        pull_card_matching(ctx, |id| {
                            clean_gwent_name(ctx.catalog.cards[id as usize].name) == name
                        })
                    {
                        ctx.core.table.push(gcard);
                        ctx.emit(EventData::CardUsed(crate::machine::card_use(
                            gcard,
                            m.owner,
                            Vec::new(),
                        )));
                        m.stage = HostStage::HuanrenPick;
                        return Flow::Yield;
                    }
                }
            }
            finish_gwent_card(m, ctx, true)
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}
