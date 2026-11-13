//! L28 雷霆万钧 Batch B（武将 10–14，`雷霆万钧/extension.js`）。
//!
//! 覆盖武将：
//! 10. `th_majun`（马钧）：`th_jingxie`、`th_qiaosi`
//! 11. `th_zhouqun`（周群）：`th_tiansuan`、`th_tiansuan2*`
//! 12. `th_wanglang`（王朗）：`th_gushe`、`th_gushe2`、`rejici`（见 `l28_batch_b_gushe.rs`）
//! 13. `th_liuhui`（刘徽）：`th_geyuan`、`th_jieshu`、`th_gusuan`（见 `l28_batch_b_geyuan.rs`）
//! 14. `th_shen_zhouyu`（神周瑜）：`th_yeyan`、`qinyin`（见 `l28_batch_b_asks.rs`）
//!
//! 小游戏（水转百戏图、命运签抽取）以玩家直接选择结果代替动画交互；其余按上游
//! `content`/`filter` 逐项实现。

use super::super::*;
use super::l28_batch_a::{
    alive_all, alive_others, discard_cards, flag, grant_skill_named, hand_and_equips,
    move_cards_to_hand, refund_once_used, set_flag,
};
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l28_batch_b_asks.rs"]
mod l28_batch_b_asks;
pub(in crate::content::host) use l28_batch_b_asks::ask_l28_batch_b;
use l28_batch_b_asks::{step_qinyin, step_th_yeyan, wait_ask};

#[path = "l28_batch_b_gushe.rs"]
mod l28_batch_b_gushe;
use l28_batch_b_gushe::{step_rejici, step_th_gushe};

#[path = "l28_batch_b_geyuan.rs"]
mod l28_batch_b_geyuan;
use l28_batch_b_geyuan::{
    geyuan_candidates, jieshu_filter, step_th_geyuan, step_th_gusuan, step_th_jieshu,
};

pub(super) const ASK_JINGXIE_CARD: u32 = 200;
pub(super) const ASK_QIAOSI_COUNT: u32 = 210;
pub(super) const ASK_QIAOSI_MODE: u32 = 211;
pub(super) const ASK_QIAOSI_TARGET: u32 = 212;
pub(super) const ASK_TIANSUAN_LOT: u32 = 220;
pub(super) const ASK_TIANSUAN_TARGET: u32 = 221;
pub(super) const ASK_TIANSUAN_GAIN: u32 = 222;
pub(super) const ASK_GUSHE_TARGETS: u32 = 230;
pub(super) const ASK_GUSHE_OWNER_CARD: u32 = 232;
pub(super) const ASK_GUSHE_TARGET_CARD: u32 = 233;
pub(super) const ASK_GUSHE_TARGET_DISCARD: u32 = 234;
pub(super) const ASK_GUSHE_OWNER_DISCARD: u32 = 235;
pub(super) const ASK_REJICI_KILLER_DISCARD: u32 = 236;
pub(super) const ASK_GEYUAN_MAGICA_TARGETS: u32 = 242;
pub(super) const ASK_GEYUAN_MAGICA_DISCARD: u32 = 243;
pub(super) const ASK_YEYAN_MODE: u32 = 250;
pub(super) const ASK_YEYAN_DISCARD: u32 = 251;
pub(super) const ASK_YEYAN_TARGETS: u32 = 252;
pub(super) const ASK_QINYIN_MODE: u32 = 260;

/// 取消即放弃整个技能（尚未产生任何效果）：结束并退还本回合次数。
/// 其余可取消询问（鼓舌「弃一张或令其摸牌」）由各技能自行把取消解释为另一选项。
const ABORT_ON_CANCEL_ASKS: [u32; 8] = [
    ASK_JINGXIE_CARD,
    ASK_QIAOSI_COUNT,
    ASK_TIANSUAN_LOT,
    ASK_GUSHE_TARGETS,
    ASK_YEYAN_MODE,
    ASK_YEYAN_TARGETS,
    ASK_YEYAN_DISCARD,
    ASK_QINYIN_MODE,
];

pub(super) fn has_skill(core: &GameStateCore, catalog: &Catalog, p: PlayerId, name: &str) -> bool {
    catalog
        .skill_by_name(name)
        .and_then(|def| core.skills.get(&(p, def)))
        .is_some_and(|inst| inst.enabled)
}

pub(super) fn revoke_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.remove(&(p, def));
    }
}

/// 本回合唯一戳（轮次 × 8 + 当前回合角色 + 1）：按回合计数的标记据此判断是否过期。
pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    core.turn.round * 8 + u32::from(core.turn.current) + 1
}

pub(super) fn card_rank(catalog: &Catalog, c: CardId) -> u32 {
    catalog
        .cards
        .get(c as usize)
        .map(|cd| u32::from(cd.rank))
        .unwrap_or(0)
}

/// 防具牌或【诸葛连弩】（`extension.js:7429` 精械可展示的牌）。
fn jingxie_upgradable(catalog: &Catalog, c: CardId) -> bool {
    let kind = catalog.card_kind(c);
    kind.equip_slot() == Some(EquipSlot::Armor) || kind == CardKind::Zhuge
}

/// 精械候选：出牌阶段为未升级过的防具/诸葛连弩；濒死时为可重铸的防具牌。
pub(super) fn jingxie_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    dying: bool,
) -> Vec<CardId> {
    hand_and_equips(core, owner)
        .into_iter()
        .filter(|&c| {
            if dying {
                catalog.card_kind(c).equip_slot() == Some(EquipSlot::Armor)
            } else {
                jingxie_upgradable(catalog, c)
                    && flag(core, owner, &format!("th_jingxie_upgraded:{c}")) == 0
            }
        })
        .collect()
}

pub(in crate::content::host) fn trigger_meta_l28_batch_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "th_jingxie" => Some(meta(EventKind::Dying, Vec::new())),
        "th_tiansuan2" => Some(meta(EventKind::TurnBegin, Vec::new())),
        "th_tiansuan2_damage" => Some(meta(EventKind::Damage, Vec::new())),
        // 中签：伤害前（`damageBefore`）改为火属性。
        "th_tiansuan2_fire" => Some(meta(EventKind::DamageBefore, Vec::new())),
        "rejici" => Some(meta(EventKind::Death, Vec::new())),
        "th_geyuan" => Some(meta(
            EventKind::CardsDiscarded,
            vec![
                EventKind::CardsLost,
                EventKind::CardUsed,
                EventKind::CardResponded,
                EventKind::TurnBegin,
            ],
        )),
        "th_jieshu" => Some(TriggerMeta {
            event: EventKind::CardUsed,
            timing: Timing::Normal,
            priority: 2,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::CardResponded],
        }),
        "th_gusuan" => Some(meta(EventKind::TurnPhaseEnd, Vec::new())),
        "qinyin" | "th_qinyin" => {
            Some(meta(EventKind::PhaseEnd, vec![EventKind::CardsDiscarded]))
        }
        "th_tianming" => Some(meta(EventKind::CardUsed, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l28_batch_b(name: &str) -> SkillLimits {
    match name {
        // 巧思（`usable: 1`）、天算（命运签存续期间不可再抽）、业炎（限定技）。
        "th_qiaosi" | "th_tiansuan" | "th_yeyan" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

fn discard_phase_count(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "qinyin_discard_stamp") == turn_stamp(core) {
        flag(core, owner, "qinyin_discard_count")
    } else {
        0
    }
}

pub(in crate::content::host) fn filter_l28_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    // 激词②（`rejici2`，forceDie）：死亡时仍结算。
    if skill.name == "rejici" {
        return matches!(
            event,
            EventData::Death { player, killer: Some(k) }
                if *player == owner && *k != owner && ctx.state.player(*k).alive
        );
    }
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        // 精械②：进入濒死时可重铸一张防具牌。
        "th_jingxie" => {
            matches!(event, EventData::Dying { player, .. } if *player == owner)
                && !jingxie_candidates(ctx.state, ctx.catalog, owner, true).is_empty()
        }
        "th_tiansuan2" => matches!(
            event,
            EventData::TurnBegin { player }
                if *player == owner && flag(ctx.state, owner, "th_tiansuan2_active") > 0
        ),
        "th_tiansuan2_damage" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0
        ),
        "th_tiansuan2_fire" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner && !d.prevented && d.nature != DamageNature::Fire
        ),
        "th_geyuan" => match event {
            EventData::TurnBegin { .. } => {
                flag(ctx.state, owner, "th_geyuan_ring_len") == 0
                    && flag(ctx.state, owner, "th_gusuan_awakened") == 0
            }
            _ => !geyuan_candidates(ctx.state, ctx.catalog, owner, event).is_empty(),
        },
        "th_jieshu" => jieshu_filter(ctx.state, ctx.catalog, owner, event),
        // 孤算（觉醒技，`extension.js:8614`）：一名角色的回合结束时，「圆环之理」长度为 3。
        "th_gusuan" => matches!(event, EventData::TurnEndWindow { after: false, .. })
            && flag(ctx.state, owner, "th_gusuan_awakened") == 0
            && flag(ctx.state, owner, "th_geyuan_ring_len") == 3,
        // 琴音（`extension.js` 引用 `character/extra.js:8906`）：弃牌阶段结束时，
        // 若本阶段弃置了至少两张牌。弃牌事件只用于计数。
        "qinyin" | "th_qinyin" => match event {
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == Phase::Discard
            }
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: false,
            } => *player == owner && discard_phase_count(ctx.state, owner) >= 2,
            _ => false,
        },
        "th_tianming" => matches!(
            event,
            EventData::CardUsed(u)
                if u.targets.contains(&owner) && u.effective_kind(ctx.catalog).is_sha_like()
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_l28_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "th_jingxie" => !jingxie_candidates(ctx.state, ctx.catalog, owner, false).is_empty(),
        "th_qiaosi" => true,
        "th_tiansuan" => flag(ctx.state, owner, "th_tiansuan2_active") == 0,
        "th_gushe" => l28_batch_b_gushe::gushe_can_activate(ctx.state, owner),
        "th_yeyan" => flag(ctx.state, owner, "th_yeyan_used") == 0,
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L28ThunderBatchB,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l28_batch_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    match event {
        EventData::Dying { player, .. } => target = Some(*player),
        EventData::TurnBegin { player } => target = Some(*player),
        EventData::Death { killer, .. } => target = *killer,
        EventData::Phase { .. } => target = Some(owner),
        EventData::Damage(d) => {
            target = Some(d.target);
            card = d.card;
            cards = vec![0; d.amount.max(0) as usize];
        }
        EventData::CardUsed(u) => {
            target = Some(u.player);
            card = u.card;
            cards = u.card.into_iter().collect();
        }
        EventData::CardResponded { player, card: c } => {
            target = Some(*player);
            card = Some(*c);
            cards = vec![*c];
        }
        EventData::CardsDiscarded { player, cards: disc } => {
            target = Some(*player);
            cards = disc.clone();
        }
        EventData::CardsLost {
            player, cards: lost, ..
        } => {
            target = Some(*player);
            cards = lost.clone();
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L28ThunderBatchB,
        owner,
        target,
        card,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l28_batch_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act
        && ctx.answer().is_empty()
        && ABORT_ON_CANCEL_ASKS.contains(&flag(ctx.core, owner, "l28_ask_id"))
    {
        // 放弃发动：退还次数，不产生任何效果。
        refund_once_used(ctx, owner, &m.def_key);
        set_flag(ctx.core, owner, "l28_ask_id", 0);
        return Flow::Done;
    }
    set_flag(ctx.core, owner, &format!("th_skill:{}", m.def_key), 1);
    match m.def_key.as_str() {
        "th_jingxie" => step_th_jingxie(m, ctx),
        "th_qiaosi" => step_th_qiaosi(m, ctx),
        "th_tiansuan" => step_th_tiansuan(m, ctx),
        "th_tiansuan2" => {
            clear_tiansuan_lots(ctx, owner);
            Flow::Done
        }
        // 上签附带：受到伤害后，摸等同于伤害值的牌（`extension.js:8104`，摸牌者为签的持有者）。
        "th_tiansuan2_damage" => {
            let n = m.cards.len() as u32;
            if n > 0 {
                ctx.start_draw(owner, n);
            }
            Flow::Done
        }
        "th_tiansuan2_fire" => {
            let id = ctx.current_event_frame_id();
            if let Some(frame) = ctx.res.frame_mut(id) {
                if let EventData::Damage(d) = &mut frame.data {
                    d.nature = DamageNature::Fire;
                }
            }
            Flow::Done
        }
        "th_gushe" => step_th_gushe(m, ctx),
        "rejici" => step_rejici(m, ctx),
        "th_geyuan" => step_th_geyuan(m, ctx),
        "th_jieshu" => step_th_jieshu(m, ctx),
        "th_gusuan" => step_th_gusuan(m, ctx),
        "th_yeyan" => step_th_yeyan(m, ctx),
        "qinyin" | "th_qinyin" => step_qinyin(m, ctx),
        "th_tianming" => {
            let to_disc: Vec<CardId> = hand_and_equips(ctx.core, owner).into_iter().take(2).collect();
            discard_cards(ctx, owner, &to_disc);
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// 精械（`extension.js:7429`）：①出牌阶段展示一张防具或【诸葛连弩】升级之（每张一次）；
/// ②进入濒死时可重铸一张防具牌，然后体力回复至 1 点。
fn step_th_jingxie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_JINGXIE_CARD),
        HostStage::SelectCard => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Card(c)) = ans.first().copied() else {
                return Flow::Done;
            };
            if m.target == Some(owner) {
                discard_cards(ctx, owner, &[c]);
                ctx.start_draw(owner, 1);
                let hp = ctx.core.player(owner).hp;
                if hp < 1 {
                    ctx.recover(owner, (1 - hp) as u32, None);
                }
                set_flag(ctx.core, owner, "th_jingxie_saved_to_1hp", 1);
            } else {
                set_flag(ctx.core, owner, &format!("th_jingxie_upgraded:{c}"), 1);
                let upg = flag(ctx.core, owner, "th_jingxie_upgraded") + 1;
                set_flag(ctx.core, owner, "th_jingxie_upgraded", upg);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_th_qiaosi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_QIAOSI_COUNT),
        HostStage::SelectNature => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Tag(cnt)) = ans.first().copied() else {
                return Flow::Done;
            };
            let n = u32::from(cnt).clamp(1, 3);
            set_flag(ctx.core, owner, "th_qiaosi_won_count", n);
            set_flag(ctx.core, owner, "th_qiaosi_drawn", n);
            set_flag(ctx.core, owner, "th_qiaosi_triggered", 1);
            let mut gained = 0u32;
            for c in ctx.core.deck.clone() {
                if gained >= n {
                    break;
                }
                let kind = ctx.catalog.card_kind(c);
                if kind.equip_slot().is_some()
                    || !kind.is_basic_kind()
                    || matches!(
                        kind,
                        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                {
                    ctx.core.deck.retain(|&x| x != c);
                    ctx.core.player_mut(owner).hand.push(c);
                    gained += 1;
                }
            }
            if gained < n {
                ctx.start_draw(owner, n - gained);
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_QIAOSI_MODE)
        }
        HostStage::SelectCard => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Tag(mode)) = ans.first().copied() else {
                return Flow::Done;
            };
            let n = flag(ctx.core, owner, "th_qiaosi_won_count").clamp(1, 3) as usize;
            if mode == 1 && !alive_others(ctx.core, owner).is_empty() {
                set_flag(ctx.core, owner, "th_qiaosi_give_other", 1);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIAOSI_TARGET)
            } else {
                let to_disc: Vec<CardId> =
                    hand_and_equips(ctx.core, owner).into_iter().take(n).collect();
                let cnt = to_disc.len() as u32;
                discard_cards(ctx, owner, &to_disc);
                set_flag(ctx.core, owner, "th_qiaosi_discarded", cnt);
                Flow::Done
            }
        }
        HostStage::TargetChoice => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Player(t)) = ans.first().copied() else {
                return Flow::Done;
            };
            let n = flag(ctx.core, owner, "th_qiaosi_won_count").clamp(1, 3) as usize;
            let to_give: Vec<CardId> =
                hand_and_equips(ctx.core, owner).into_iter().take(n).collect();
            let cnt = to_give.len() as u32;
            move_cards_to_hand(ctx, owner, t, &to_give);
            set_flag(ctx.core, owner, "th_qiaosi_given", cnt.max(1));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

const TIANSUAN_LOT_SKILLS: [&str; 5] = [
    "th_tiansuan2_0",
    "th_tiansuan2_1",
    "th_tiansuan2_2",
    "th_tiansuan2_3",
    "th_tiansuan2_4",
];

/// 命运签效果标记（由 `rvm_bridge/modifiers/d34_flags.rs` 读取的伤害修饰）。
const TIANSUAN_LOT_FLAGS: [&str; 6] = [
    "th_tiansuan2_0_prevent_damage",
    "th_tiansuan2_1_cap_damage",
    "th_tiansuan2_2_cap_damage",
    "th_tiansuan2_3_extra_damage",
    "th_tiansuan2_4_extra_damage",
    "th_tiansuan2_4_forbid_tao_jiu",
];

/// `th_tiansuan2.onremove`（`extension.js:8027`）：周群下个回合开始时移除目标的命运签。
fn clear_tiansuan_lots(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let prev_t_raw = flag(ctx.core, owner, "th_tiansuan_target");
    if prev_t_raw > 0 && (prev_t_raw as usize - 1) < ctx.core.players.len() {
        let pt = (prev_t_raw - 1) as PlayerId;
        for k in TIANSUAN_LOT_FLAGS {
            set_flag(ctx.core, pt, k, 0);
        }
        for sk in TIANSUAN_LOT_SKILLS
            .iter()
            .chain(["th_tiansuan2_damage", "th_tiansuan2_fire", "th_tiansuan2_ai"].iter())
        {
            revoke_skill_named(ctx, pt, sk);
        }
    }
    set_flag(ctx.core, owner, "th_tiansuan2_active", 0);
    set_flag(ctx.core, owner, "th_tiansuan_lot", 0);
    set_flag(ctx.core, owner, "th_tiansuan_target", 0);
    revoke_skill_named(ctx, owner, "th_tiansuan2");
}

/// 天算获得牌的范围：目标为自己时只取装备区（`target == player ? 'e' : 'he'`）。
pub(super) fn tiansuan_gain_pool(core: &GameStateCore, owner: PlayerId, t: PlayerId) -> Vec<CardId> {
    if t == owner {
        core.player(t).equips.clone()
    } else {
        hand_and_equips(core, t)
    }
}

/// 天算（`extension.js:7806-8026`）：抽取命运签（玩家直接选择签）→ 选择一名角色获得签，
/// 直到周群下个回合开始；上上签/上签时周群从其区域获得一张牌。
fn step_th_tiansuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_TIANSUAN_LOT),
        HostStage::SelectNature => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Tag(lot)) = ans.first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "th_tiansuan_pending_lot", u32::from(lot % 5) + 1);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_TIANSUAN_TARGET)
        }
        HostStage::TargetChoice => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Player(t)) = ans.first().copied() else {
                return Flow::Done;
            };
            let idx = flag(ctx.core, owner, "th_tiansuan_pending_lot").saturating_sub(1) % 5;
            set_flag(ctx.core, owner, "th_tiansuan_pending_lot", 0);
            set_flag(ctx.core, owner, "th_tiansuan_lot", idx + 1);
            set_flag(ctx.core, owner, "th_tiansuan2_active", 1);
            set_flag(ctx.core, owner, "th_tiansuan_target", u32::from(t) + 1);
            grant_skill_named(ctx, owner, "th_tiansuan2");
            grant_skill_named(ctx, t, TIANSUAN_LOT_SKILLS[idx as usize]);
            match idx {
                0 => set_flag(ctx.core, t, "th_tiansuan2_0_prevent_damage", 1),
                1 => {
                    set_flag(ctx.core, t, "th_tiansuan2_1_cap_damage", 1);
                    grant_skill_named(ctx, t, "th_tiansuan2_damage");
                }
                2 => {
                    set_flag(ctx.core, t, "th_tiansuan2_2_cap_damage", 1);
                    grant_skill_named(ctx, t, "th_tiansuan2_fire");
                    grant_skill_named(ctx, t, "th_tiansuan2_ai");
                }
                3 => set_flag(ctx.core, t, "th_tiansuan2_3_extra_damage", 1),
                _ => {
                    set_flag(ctx.core, t, "th_tiansuan2_4_extra_damage", 1);
                    set_flag(ctx.core, t, "th_tiansuan2_4_forbid_tao_jiu", 1);
                }
            }
            if idx < 2 && !tiansuan_gain_pool(ctx.core, owner, t).is_empty() {
                m.target = Some(t);
                return wait_ask(m, ctx, HostStage::SelectCard, ASK_TIANSUAN_GAIN);
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            let ans = ctx.answer().clone();
            let (Some(ChoiceOption::Card(c)), Some(t)) = (ans.first().copied(), m.target) else {
                return Flow::Done;
            };
            if t == owner {
                if let Some(pos) = ctx.core.player(owner).equips.iter().position(|&x| x == c) {
                    ctx.core.player_mut(owner).equips.remove(pos);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            } else {
                move_cards_to_hand(ctx, t, owner, &[c]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
