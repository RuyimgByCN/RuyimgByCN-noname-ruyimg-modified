//! L11（太虚幻境）全部装备牌技能及关联子状态（`l11_equips.rs`）：
//!
//! - `txhj_xingtianpojunfu` / `txhj_noda_axe` / `txhj_noda_axe2`（`extension_card.js:2238-2285`）
//! - `txhj_zhuge`（`extension_card.js:1554` — 修正由 `d34_flags.rs` 执行）
//! - `txhj_qilingong` / `qilin_skill`（`extension_card.js:1673` & `extension_skill.js:856`）
//! - `txhj_fangtianhuaji`（`extension_card.js:1639`）
//! - `txhj_xuwangzhimian`（`extension_card.js:1700` — 摸牌 +2 由 `l11_common::on_draw_begin_l11`，手牌上限 -1 由 `d34_flags.rs`）
//! - `txhj_piliche` / `txhj_piliche2`（`extension_card.js:1716`；离场销毁由 `machine.rs`）
//! - `txhj_liulongcanjia`（`extension_card.js:1733` — 互相替换由 `equipment.rs`，距离与不可置入其他坐骑由 `d34_l11_mods.rs`）
//! - `txhj_leijimu`（`extension_card.js:1743`）
//! - `txhj_juechenjinge` / `txhj_juechenjinge2`（`extension_card.js:1781` — 距离由 `d34_l11_mods.rs`）
//! - `txhj_guofengyupao`（`extension_card.js:1798` — 普通锦囊免疫由 `d34_l11_mods.rs`）
//! - `txhj_feilong` / `txhj_feilong_skill` / `txhj_feilong_skill_blank`（`extension_card.js:1815`）
//! - `txhj_dinglanyemingzhu`（`extension_card.js:1850`）
//! - `txhj_chiyanzhenhunqin`（`extension_card.js:1868`）
//! - `txhj_lingbaoxianhu`（`extension_card.js:1880`）
//! - `txhj_chongyingshenfu` / `txhj_chongyingshenfu_effect`（`extension_card.js:2339-2381`）
//! - `txminguangkai` / `txminguangkai_cancel` / `txminguangkai_link`（`extension_card.js:1593-1637`）
//! - `txhj_taijifuchen` / `gx_taijifuchen`（`extension_skill.js:605-660` & `extension_card.js:2429`）
//! - `txhj_baihu`（`extension_card.js:1263` — 距离 +2 由 `d34_l11_mods.rs`）

use super::l11_common::*;
use super::l11_equips_asks::*;
use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};
use crate::rvm_bridge::modifiers::{d34_l28_mods, is_target_unequip2};

fn wait_eq_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_equip_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        Flow::Done
    }
}

pub(in crate::content::host) fn trigger_meta_l11_equips(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, optional: bool, extra_events| {
        TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional,
            extra_events,
        }
    };
    match name {
        "txhj_xingtianpojunfu" | "txhj_noda_axe" => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 1, false, Vec::new()))
        }
        "txhj_qilingong" | "qilin_skill" => Some(meta(
            EventKind::DamageBegin2,
            Timing::Normal,
            1,
            false,
            Vec::new(),
        )),
        "txhj_fangtianhuaji" => Some(meta(
            EventKind::CardUsed,
            Timing::First,
            12,
            false,
            Vec::new(),
        )),
        "txhj_piliche" => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            1,
            false,
            Vec::new(),
        )),
        "txhj_leijimu" => Some(meta(
            EventKind::CardUsed,
            Timing::First,
            10,
            false,
            Vec::new(),
        )),
        "txhj_feilong" | "txhj_feilong_skill" => {
            Some(meta(EventKind::Death, Timing::Last, -25, false, Vec::new()))
        }
        "txhj_feilong_skill_blank" => Some(meta(
            EventKind::DamageAfter,
            Timing::Normal,
            1,
            false,
            Vec::new(),
        )),
        "txhj_dinglanyemingzhu" => Some(meta(
            EventKind::TurnEnded,
            Timing::Normal,
            1,
            false,
            Vec::new(),
        )),
        "txhj_chiyanzhenhunqin" => Some(meta(
            EventKind::DamageBegin1,
            Timing::Normal,
            5,
            false,
            Vec::new(),
        )),
        "txhj_lingbaoxianhu" => Some(meta(
            EventKind::DamageSource,
            Timing::Normal,
            1,
            false,
            vec![EventKind::Death],
        )),
        "txhj_chongyingshenfu" => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            2,
            false,
            Vec::new(),
        )),
        "txhj_chongyingshenfu_effect" => Some(meta(
            EventKind::DamageBegin4,
            Timing::Normal,
            1,
            false,
            Vec::new(),
        )),
        "txminguangkai" | "txminguangkai_cancel" => Some(meta(
            EventKind::CardUsed,
            Timing::First,
            8,
            false,
            Vec::new(),
        )),
        "txhj_taijifuchen" | "gx_taijifuchen" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            2,
            false,
            Vec::new(),
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_equips(name: &str) -> SkillLimits {
    match name {
        "txhj_dinglanyemingzhu" => SkillLimits {
            once_per_turn: true,
            active_outside_play: false,
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_l11_equips(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    if skill.name != "txhj_dinglanyemingzhu" {
        return None;
    }
    let state = ctx.state;
    if state.turn.phase != Phase::Play
        || !is_unblocked_l11_owner(state, ctx.catalog, owner)
        || has_named_skill(state, ctx.catalog, owner, "gzzhiheng")
        || state.player(owner).max_hp == 0
    {
        return None;
    }
    if mingzhu_discardable_cards(state, ctx.catalog, owner).is_empty() {
        return None;
    }
    active_skill_machine("txhj_dinglanyemingzhu", HostBehavior::L11EquipCards, owner)
}

fn is_minguangkai_fire_card(catalog: &Catalog, u: &crate::event::CardUseData) -> bool {
    let eff = effective_kind_of_use(catalog, u);
    if eff == Some(CardKind::FireSha) {
        return true;
    }
    if let Some(c) = u.card {
        if let Some(cd) = catalog.cards.get(c as usize) {
            if matches!(
                cd.name,
                "huoshaolianying" | "txhj_huoshaolianying" | "huogong"
            ) && u.use_as.is_none()
            {
                return true;
            }
        }
    }
    false
}

pub(in crate::content::host) fn filter_l11_equips(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "txhj_xingtianpojunfu" | "txhj_noda_axe" => {
            if skill.name == "txhj_noda_axe"
                && has_equip_named(state, catalog, owner, "txhj_xingtianpojunfu")
            {
                return false;
            }
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player == owner
                && !u.cancelled
                && state.turn.current == owner
                && state.turn.phase == Phase::Play
                && u.targets.len() == 1
                && u.targets[0] != owner
                && state.player(u.targets[0]).alive
                && !has_named_skill(state, catalog, u.targets[0], "txhj_noda_axe2")
                && axe_discardable_cards(state, catalog, owner).len() >= 2
        }
        "txhj_qilingong" | "qilin_skill" => {
            if skill.name == "qilin_skill"
                && has_equip_named(state, catalog, owner, "txhj_qilingong")
            {
                return false;
            }
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && !d.prevented
                && !d.from_chain
                && d.amount > 0
                && d.card.is_some_and(|c| catalog.card_kind(c).is_sha_like())
                && state.player(d.target).alive
                && equip_slot_card(state, catalog, d.target, EquipSlot::Treasure).is_some()
        }
        "txhj_fangtianhuaji" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player == owner
                && !u.cancelled
                && state.turn.current == owner
                && state.turn.phase == Phase::Play
                && state.alive_count() >= 3
                && is_sha_use(catalog, u)
                && flag(state, owner, "l11_phase_sha_count") <= 1
                && !fangtian_extra_candidates(
                    state,
                    catalog,
                    owner,
                    &u.targets,
                    u.card,
                    effective_kind_of_use(catalog, u).unwrap_or(CardKind::Sha),
                    u.nature,
                )
                .is_empty()
        }
        "txhj_piliche" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && d.target != owner
                && !d.prevented
                && d.amount > 0
                && state.player(d.target).alive
                && state.player(d.target).max_hp > 0
                && !state.player(d.target).infinite_hp()
        }
        "txhj_leijimu" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player == owner
                && !u.cancelled
                && u.nature == DamageNature::Normal
                && effective_kind_of_use(catalog, u) == Some(CardKind::Sha)
        }
        "txhj_feilong" | "txhj_feilong_skill" => {
            if skill.name == "txhj_feilong_skill"
                && has_equip_named(state, catalog, owner, "txhj_feilong")
            {
                return false;
            }
            let EventData::Death { player, killer } = event else {
                return false;
            };
            *killer == Some(owner)
                && *player != owner
                && !would_death_end_game(state, catalog, *player)
        }
        "txhj_feilong_skill_blank" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner && !d.prevented && d.amount > 0
        }
        "txhj_chiyanzhenhunqin" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner) && !d.prevented && d.amount > 0
        }
        "txhj_lingbaoxianhu" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner) && !d.prevented && d.amount > 1
            }
            EventData::Death { player, .. } => *player != owner,
            _ => false,
        },
        "txhj_chongyingshenfu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.target != owner || d.prevented || d.amount <= 0 || is_target_unequip2(state, owner)
            {
                return false;
            }
            if d.source.is_some_and(|s| {
                d34_l28_mods::source_ignores_armor(
                    state, catalog, s, owner, d.card, None, d.nature,
                )
            }) {
                return false;
            }
            let Some(cname) = d.card.and_then(|c| catalog.cards.get(c as usize)).map(|cd| cd.name)
            else {
                return false;
            };
            let key = format!("l11_chongying_rec:{cname}");
            if flag(state, owner, &key) == 0 {
                return true;
            }
            // Also decrement DamageBegin4 if `txhj_chongyingshenfu_effect` wasn't separately added as a child
            false
        }
        "txhj_chongyingshenfu_effect" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.target != owner || d.prevented || d.amount <= 0 || is_target_unequip2(state, owner)
            {
                return false;
            }
            if d.source.is_some_and(|s| {
                d34_l28_mods::source_ignores_armor(
                    state, catalog, s, owner, d.card, None, d.nature,
                )
            }) {
                return false;
            }
            d.card
                .and_then(|c| catalog.cards.get(c as usize))
                .is_some_and(|cd| flag(state, owner, &format!("l11_chongying_rec:{}", cd.name)) > 0)
        }
        "txminguangkai" | "txminguangkai_cancel" => {
            if skill.name == "txminguangkai_cancel"
                && has_equip_named(state, catalog, owner, "txminguangkai")
            {
                return false;
            }
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.targets.contains(&owner)
                && !u.cancelled
                && is_minguangkai_fire_card(catalog, u)
                && !is_target_unequip2(state, owner)
                && !d34_l28_mods::source_ignores_armor(
                    state,
                    catalog,
                    u.player,
                    owner,
                    u.card,
                    effective_kind_of_use(catalog, u),
                    u.nature,
                )
        }
        "txhj_taijifuchen" | "gx_taijifuchen" => {
            if skill.name == "gx_taijifuchen"
                && has_equip_named(state, catalog, owner, "txhj_taijifuchen")
            {
                return false;
            }
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player == owner && !u.cancelled && is_sha_use(catalog, u) && !u.targets.is_empty()
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_equips(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut cards = Vec::new();
    let mut target = None;
    let mut targets = Vec::new();
    match event {
        EventData::CardUsed(u) => {
            if let Some(cid) = u.card {
                cards.push(cid);
            }
            targets = u.targets.clone();
            target = u.targets.first().copied();
        }
        EventData::Damage(d) => {
            target = Some(d.target);
            cards.push(d.amount.max(0) as u32);
            if let Some(c) = d.card {
                cards.push(c);
            }
        }
        EventData::Death { player, .. } => {
            target = Some(*player);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11EquipCards,
        owner,
        target,
        card: None,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_equips(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_xingtianpojunfu" | "txhj_noda_axe" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                if has_named_skill(ctx.core, ctx.catalog, t, "txhj_noda_axe2")
                    || axe_discardable_cards(ctx.core, ctx.catalog, owner).len() < 2
                {
                    return Flow::Done;
                }
                let ask = if m.def_key == "txhj_xingtianpojunfu" {
                    ASK_NODA_AXE_DISCARD
                } else {
                    ASK_NODA_AXE_RAW_DISCARD
                };
                wait_eq_ask(m, ctx, HostStage::SelectCard, ask)
            }
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if chosen.len() == 2 {
                    if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                        discard_player_cards(ctx, owner, &chosen);
                        ensure_perm_skill(ctx, t, "txhj_noda_axe2");
                        set_flag(ctx.core, t, "l11_noda_axe2_turn", 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_qilingong" | "qilin_skill" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                if equip_slot_card(ctx.core, ctx.catalog, t, EquipSlot::Treasure).is_none() {
                    return Flow::Done;
                }
                let ask = if m.def_key == "txhj_qilingong" {
                    ASK_QILIN_CONFIRM
                } else {
                    ASK_QILIN_SKILL_CONFIRM
                };
                wait_eq_ask(m, ctx, HostStage::TargetChoice, ask)
            }
            HostStage::TargetChoice | HostStage::SelectCard => {
                if is_confirmed(ctx) {
                    if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                        let treasures: Vec<CardId> = ctx
                            .core
                            .player(t)
                            .equips
                            .iter()
                            .copied()
                            .filter(|&c| {
                                ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Treasure)
                            })
                            .collect();
                        let second_eligible = m.stage == HostStage::TargetChoice
                            && treasures.len() > 1
                            && crate::rvm_bridge::modifiers::has_fuchendangmo(
                                ctx.core,
                                ctx.catalog,
                                owner,
                            );
                        if let Some(tr) = treasures.first().copied() {
                            discard_player_cards(ctx, t, &[tr]);
                            if crate::rvm_bridge::modifiers::has_shenhanxueyin(
                                ctx.core,
                                ctx.catalog,
                                owner,
                            ) && ctx.core.discard.contains(&tr)
                            {
                                gain_card_from_deck_or_discard(ctx, owner, tr);
                            }
                            if second_eligible && ctx.core.player(t).alive {
                                let ask = if m.def_key == "txhj_qilingong" {
                                    ASK_QILIN_CONFIRM
                                } else {
                                    ASK_QILIN_SKILL_CONFIRM
                                };
                                return wait_eq_ask(m, ctx, HostStage::SelectCard, ask);
                            }
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_fangtianhuaji" => match m.stage {
            HostStage::Act => {
                let fid = ctx.current_event_frame_id();
                if (flag(ctx.core, owner, &format!("l11_first_sha_frame:{fid}")) == 0
                    && flag(ctx.core, owner, "l11_phase_sha_count") > 1)
                    || ctx.core.alive_count() < 3
                {
                    return Flow::Done;
                }
                let sha_cid = m.cards.first().copied();
                let (sha_kind, sha_nature) = if let Some(EventData::CardUsed(u)) =
                    ctx.res.frame(fid).map(|f| &f.data)
                {
                    (
                        effective_kind_of_use(ctx.catalog, u).unwrap_or(CardKind::Sha),
                        u.nature,
                    )
                } else {
                    let k = sha_cid
                        .map(|c| ctx.catalog.card_kind(c))
                        .unwrap_or(CardKind::Sha);
                    let n = match k {
                        CardKind::FireSha => DamageNature::Fire,
                        CardKind::LeiSha => DamageNature::Thunder,
                        _ => DamageNature::Normal,
                    };
                    (k, n)
                };
                set_fangtian_meta(ctx.core, owner, sha_kind, sha_nature);
                let cands = fangtian_extra_candidates(
                    ctx.core,
                    ctx.catalog,
                    owner,
                    &m.targets,
                    sha_cid,
                    sha_kind,
                    sha_nature,
                );
                if cands.is_empty() {
                    clear_fangtian_meta(ctx.core, owner);
                    return Flow::Done;
                }
                wait_eq_ask(m, ctx, HostStage::TargetChoice, ASK_FANGTIAN_TARGETS)
            }
            HostStage::TargetChoice => {
                clear_fangtian_meta(ctx.core, owner);
                let chosen: Vec<PlayerId> = answered_players(ctx)
                    .into_iter()
                    .take(2)
                    .filter(|&p| ctx.core.player(p).alive)
                    .collect();
                if !chosen.is_empty() {
                    if let Some(u) = current_use_card_mut(ctx) {
                        for p in chosen {
                            if !u.targets.contains(&p) {
                                u.targets.push(p);
                            }
                        }
                    }
                }
                Flow::Done
            }
            _ => {
                clear_fangtian_meta(ctx.core, owner);
                Flow::Done
            }
        },
        "txhj_piliche" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive && ctx.core.player(p).max_hp > 0)
                else {
                    return Flow::Done;
                };
                if ctx.core.player(t).infinite_hp() {
                    return Flow::Done;
                }
                wait_eq_ask(m, ctx, HostStage::TargetChoice, ASK_PILICHE_CONFIRM)
            }
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                        let dmg = m.cards.first().copied().unwrap_or(1).max(1);
                        ctx.lose_max_hp(t, dmg);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_leijimu" => match m.stage {
            HostStage::Act => wait_eq_ask(m, ctx, HostStage::TargetChoice, ASK_LEIJIMU_CONFIRM),
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    if let Some(u) = current_use_card_mut(ctx) {
                        u.nature = DamageNature::Thunder;
                        u.use_as = Some(CardKind::LeiSha);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_feilong" | "txhj_feilong_skill" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                if would_death_end_game(ctx.core, ctx.catalog, t) {
                    return Flow::Done;
                }
                let ask = if m.def_key == "txhj_feilong" {
                    ASK_FEILONG_CONFIRM
                } else {
                    ASK_FEILONG_SKILL_CONFIRM
                };
                wait_eq_ask(m, ctx, HostStage::TargetChoice, ask)
            }
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    if let Some(t) = m.target {
                        if !would_death_end_game(ctx.core, ctx.catalog, t) {
                            revive_as_ally_3_3(ctx, owner, t);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_feilong_skill_blank" => {
            if flag(ctx.core, owner, "l11_feilong_just_revived") > 0 {
                set_flag(ctx.core, owner, "l11_feilong_just_revived", 0);
                return Flow::Done;
            }
            clear_skill_blocked_flag_on_damage(ctx, owner);
            Flow::Done
        }
        "txhj_dinglanyemingzhu" => match m.stage {
            HostStage::Act => wait_eq_ask(m, ctx, HostStage::SelectCard, ASK_MINGZHU_DISCARD),
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if chosen.is_empty() {
                    refund_once_used(ctx, owner, "txhj_dinglanyemingzhu");
                    return Flow::Done;
                }
                let n = chosen.len() as u32;
                discard_player_cards(ctx, owner, &chosen);
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, n);
                Flow::Yield
            }
            _ => Flow::Done,
        },
        "txhj_chiyanzhenhunqin" => {
            with_current_damage_mut(ctx, |dm| {
                dm.nature = DamageNature::Fire;
            });
            Flow::Done
        }
        "txhj_lingbaoxianhu" => {
            if !ctx.core.player(owner).infinite_hp() {
                ctx.core.player_mut(owner).max_hp += 1;
            }
            ctx.recover(owner, 1, None);
            Flow::Done
        }
        "txhj_chongyingshenfu" => {
            let cur_fid = ctx.current_event_frame_id();
            let root_id = match ctx.res.frame(cur_fid).map(|fr| &fr.data) {
                Some(EventData::DamageWindow { root, .. }) => Some(*root),
                Some(EventData::Damage(_)) => Some(cur_fid),
                _ => None,
            };
            let card_id = root_id
                .and_then(|rid| ctx.res.frame(rid))
                .and_then(|fr| match &fr.data {
                    EventData::Damage(d) => d.card,
                    _ => None,
                })
                .or_else(|| m.cards.get(1).copied());
            if let Some(cname) = card_id
                .and_then(|c| ctx.catalog.cards.get(c as usize))
                .map(|cd| cd.name)
            {
                let key = format!("l11_chongying_rec:{cname}");
                if flag(ctx.core, owner, &key) == 0 {
                    set_flag(ctx.core, owner, &key, 1);
                    ensure_perm_skill(ctx, owner, "txhj_chongyingshenfu_effect");
                }
            }
            Flow::Done
        }
        "txhj_chongyingshenfu_effect" => {
            with_current_damage_mut(ctx, |dm| {
                if dm.amount > 0 {
                    dm.amount -= 1;
                    if dm.amount == 0 {
                        dm.prevented = true;
                    }
                }
            });
            Flow::Done
        }
        "txminguangkai" | "txminguangkai_cancel" => {
            if let Some(u) = current_use_card_mut(ctx) {
                u.targets.retain(|&p| p != owner);
                if u.targets.is_empty() {
                    u.cancelled = true;
                }
            }
            Flow::Done
        }
        "txhj_taijifuchen" | "gx_taijifuchen" => {
            let fid = ctx.current_event_frame_id();
            match m.stage {
                HostStage::Act => {
                    if let Some(next_t) = m.targets.first().copied() {
                        m.targets.remove(0);
                        let still_targeted = ctx
                            .res
                            .frame(fid)
                            .and_then(|f| match &f.data {
                                EventData::CardUsed(u) => {
                                    (!u.cancelled && u.targets.contains(&next_t)).then_some(())
                                }
                                _ => None,
                            })
                            .is_some();
                        if !still_targeted || !ctx.core.player(next_t).alive {
                            return handle_l11_equips(m, ctx);
                        }
                        let sha_cid = m.cards.first().copied();
                        let need = if crate::rvm_bridge::modifiers::has_fuchendangmo(
                            ctx.core,
                            ctx.catalog,
                            owner,
                        ) {
                            2
                        } else {
                            1
                        };
                        if he_cards(ctx.core, next_t).is_empty() {
                            set_flag(
                                ctx.core,
                                owner,
                                &format!("sha_unrespondable:{}:{next_t}", sha_cid.unwrap_or(0)),
                                1,
                            );
                            return handle_l11_equips(m, ctx);
                        }
                        set_flag(
                            ctx.core,
                            owner,
                            &format!("txhj_taiji_rem:{fid}:{next_t}"),
                            need,
                        );
                        m.target = Some(next_t);
                        let ask = if m.def_key == "txhj_taijifuchen" {
                            ASK_TAIJI_DISCARD
                        } else {
                            ASK_GX_TAIJI_DISCARD
                        };
                        return wait_eq_ask(m, ctx, HostStage::SelectCard, ask);
                    }
                    Flow::Done
                }
                HostStage::SelectCard => {
                    let sha_cid = m.cards.first().copied();
                    if let Some(t) = m.target {
                        let rem_key = format!("txhj_taiji_rem:{fid}:{t}");
                        let rem = flag(ctx.core, owner, &rem_key);
                        if let Some(ChoiceOption::Card(disc)) = ctx.answer().first().copied() {
                            if he_cards(ctx.core, t).contains(&disc) {
                                let same_suit = sha_cid.is_some_and(|sc| {
                                    ctx.catalog.effective_card_suit(ctx.core, owner, sc)
                                        == ctx.catalog.effective_card_suit(ctx.core, t, disc)
                                });
                                let xueyin = crate::rvm_bridge::modifiers::has_shenhanxueyin(
                                    ctx.core,
                                    ctx.catalog,
                                    owner,
                                );
                                discard_player_cards(ctx, t, &[disc]);
                                if (same_suit || xueyin) && ctx.core.discard.contains(&disc) {
                                    gain_card_from_deck_or_discard(ctx, owner, disc);
                                }
                                if rem > 1 {
                                    set_flag(ctx.core, owner, &rem_key, rem - 1);
                                    if !he_cards(ctx.core, t).is_empty() {
                                        let ask = if m.def_key == "txhj_taijifuchen" {
                                            ASK_TAIJI_DISCARD
                                        } else {
                                            ASK_GX_TAIJI_DISCARD
                                        };
                                        return wait_eq_ask(m, ctx, HostStage::SelectCard, ask);
                                    }
                                    set_flag(
                                        ctx.core,
                                        owner,
                                        &format!("sha_unrespondable:{}:{t}", sha_cid.unwrap_or(0)),
                                        1,
                                    );
                                }
                            } else {
                                set_flag(
                                    ctx.core,
                                    owner,
                                    &format!("sha_unrespondable:{}:{t}", sha_cid.unwrap_or(0)),
                                    1,
                                );
                            }
                        } else {
                            set_flag(
                                ctx.core,
                                owner,
                                &format!("sha_unrespondable:{}:{t}", sha_cid.unwrap_or(0)),
                                1,
                            );
                        }
                        set_flag(ctx.core, owner, &rem_key, 0);
                    }
                    m.stage = HostStage::Act;
                    m.target = None;
                    handle_l11_equips(m, ctx)
                }
                _ => Flow::Done,
            }
        }
        _ => Flow::Done,
    }
}
