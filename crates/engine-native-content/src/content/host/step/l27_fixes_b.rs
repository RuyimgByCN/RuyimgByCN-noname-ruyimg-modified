//! L27 `选将美化` Batch B 与 UI 钩子规则对齐补丁（`l27_fixes_b.rs`）：
//! 覆盖 `sbqingzheng`、`xingchong`、`dcxiaoxi`、`twfengji`、`twzhouzu`。

use super::l27_batch_b::{
    alive_others, discard_cards, emit_damage, finish_mahou_cast, finish_mahou_cast_stage,
    finish_renigong_draw, flag, hand_and_equips, mahou_max_x, move_cards_to_hand, set_flag,
    wait_ask as wait_ask_b, ASK_DCXIAOXI_COUNT, ASK_DCXIAOXI_GAIN_CARD, ASK_DCXIAOXI_MODE,
    ASK_DCXIAOXI_TARGET, ASK_SBQINGZHENG_GAIN_MARK, ASK_SBQINGZHENG_SUITS,
    ASK_SBQINGZHENG_TARGET, ASK_SBQINGZHENG_TARGET_SUIT, ASK_TWFENGJI_CARD, ASK_TWFENGJI_TURNS,
    ASK_TWZHOUZU_TARGET, ASK_TWZHOUZU_TARGET_DISCARD, ASK_TWZHOUZU_TURNS, ASK_XINGCHONG_DRAW,
    ASK_XINGCHONG_SHOW,
};
use super::l27_fixes::{clear_xingchong_state, has_skill_named, suit_tag};
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceOption;
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

/// `sbqingzheng` 步进（`选将美化/extension.js:4263-4838`）：选目标后再双向弃牌，并无条件判 `sbjianxiong` 获治世。
pub(super) fn step_sbqingzheng_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask_b(m, ctx, HostStage::SelectNature, ASK_SBQINGZHENG_SUITS),
        HostStage::SelectNature => {
            let mut suit_mask = 0u32;
            for opt in ctx.answer() {
                if let ChoiceOption::Tag(s) = *opt {
                    suit_mask |= 1u32 << (s & 3);
                }
            }
            let marks = flag(ctx.core, owner, "sbjianxiong_marks")
                .max(flag(ctx.core, owner, "sbjianxiong"))
                .min(2);
            let need = (3 - marks).clamp(1, 3) as usize;
            if suit_mask == 0 || suit_mask.count_ones() as usize != need {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            }
            m.card = Some(suit_mask);
            wait_ask_b(m, ctx, HostStage::TargetChoice, ASK_SBQINGZHENG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask_b(m, ctx, HostStage::SelectCard, ASK_SBQINGZHENG_TARGET_SUIT)
        }
        HostStage::SelectCard => {
            let s_tag = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => n & 3,
                _ => 0,
            };
            let suit_mask = m.card.unwrap_or(0) as u8;
            let t = m.target.unwrap_or(owner);
            let self_disc: Vec<CardId> = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| (suit_mask & (1u8 << suit_tag(cd.suit))) != 0)
                })
                .collect();
            let t_disc: Vec<CardId> = ctx
                .core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| suit_tag(cd.suit) == s_tag)
                })
                .collect();
            set_flag(
                ctx.core,
                owner,
                "sbqingzheng_self_discarded",
                self_disc.len() as u32,
            );
            set_flag(
                ctx.core,
                owner,
                "sbqingzheng_target_discarded",
                t_disc.len() as u32,
            );
            discard_cards(ctx, owner, &self_disc);
            discard_cards(ctx, t, &t_disc);
            // extension.js:4825 在数组比较下恒 false（无伤害），随后 step 5 无条件检 sbjianxiong。
            let marks =
                flag(ctx.core, owner, "sbjianxiong_marks").max(flag(ctx.core, owner, "sbjianxiong"));
            if has_skill_named(ctx.core, ctx.catalog, owner, "sbjianxiong") && marks < 2 {
                return wait_ask_b(m, ctx, HostStage::CardsAsk, ASK_SBQINGZHENG_GAIN_MARK);
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        HostStage::CardsAsk => {
            if !ctx.answer().is_empty()
                && !matches!(ctx.answer().first(), Some(ChoiceOption::Tag(0)))
            {
                let marks = (flag(ctx.core, owner, "sbjianxiong_marks")
                    .max(flag(ctx.core, owner, "sbjianxiong"))
                    + 1)
                .min(2);
                set_flag(ctx.core, owner, "sbjianxiong_marks", marks);
                set_flag(ctx.core, owner, "sbjianxiong", marks);
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn sync_xingchong_mark(core: &mut GameStateCore, catalog: &Catalog, owner: PlayerId) -> u32 {
    let tagged = core
        .vm_aux
        .flags
        .iter()
        .filter(|((p, k), v)| *p == owner && k.starts_with("xingchong_card:") && **v > 0)
        .count() as u32;
    set_flag(core, owner, "xingchong_tagged_count", tagged);
    if let Some(def) = catalog.skill_by_name("xingchong") {
        if tagged > 0 {
            core.vm_aux.marks.insert((owner, u32::from(def)), tagged as i32);
        } else {
            core.vm_aux.marks.remove(&(owner, u32::from(def)));
        }
    }
    tagged
}

fn open_xingchong_show(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId, rem: u32) -> Flow {
    if rem > 0 && !ctx.core.player(owner).hand.is_empty() {
        set_flag(ctx.core, owner, "xingchong_show_max", rem);
        return wait_ask_b(m, ctx, HostStage::SelectCard, ASK_XINGCHONG_SHOW);
    }
    set_flag(ctx.core, owner, "l27_b_ask_id", 0);
    Flow::Done
}

/// `xingchong` 步进（`选将美化/extension.js:6633-6771`）：先摸后展示、公开亮牌+可见标记、取消 `choose_draw` 仍进展示。
pub(super) fn step_xingchong_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(25) {
                let mut k = 0u32;
                for &c in &m.cards {
                    let key = format!("xingchong_card:{c}");
                    if flag(ctx.core, owner, &key) > 0 {
                        set_flag(ctx.core, owner, &key, 0);
                        k += 1;
                    }
                }
                sync_xingchong_mark(ctx.core, ctx.catalog, owner);
                if k > 0 {
                    ctx.start_draw(owner, 2 * k);
                }
                return Flow::Done;
            }
            clear_xingchong_state(ctx.core, ctx.catalog, owner);
            set_flag(
                ctx.core,
                owner,
                "xingchong_round_used",
                ctx.core.turn.round.max(1),
            );
            wait_ask_b(m, ctx, HostStage::SelectNature, ASK_XINGCHONG_DRAW)
        }
        HostStage::SelectNature => {
            let limit = (ctx.core.player(owner).max_hp as u32).min(5);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).min(limit),
                _ => 0,
            };
            let rem = limit.saturating_sub(x);
            m.card = Some(rem);
            if x > 0 {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, x);
                return Flow::Yield;
            }
            open_xingchong_show(m, ctx, owner, rem)
        }
        HostStage::DrawSettle => {
            let rem = m.card.unwrap_or(0);
            open_xingchong_show(m, ctx, owner, rem)
        }
        HostStage::SelectCard => {
            let mut revealed = Vec::new();
            for opt in ctx.answer().clone() {
                if let ChoiceOption::Card(c) = opt {
                    if ctx.core.player(owner).hand.contains(&c) {
                        set_flag(ctx.core, owner, &format!("xingchong_card:{c}"), 1);
                        revealed.push(c);
                    }
                }
            }
            sync_xingchong_mark(ctx.core, ctx.catalog, owner);
            if !revealed.is_empty() {
                ctx.log(LogEvent::CardsRevealed {
                    cards: revealed,
                    to: Vec::new(),
                });
            }
            set_flag(ctx.core, owner, "xingchong_show_max", 0);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn dcxiaoxi_in_range_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    let range = core.attack_range(catalog, owner);
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| core.computed_distance(catalog, owner, p) <= range)
        .collect()
}

fn dcxiaoxi_apply_lose_max_hp(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    x: u32,
) -> Flow {
    let p = ctx.core.player_mut(owner);
    p.max_hp = p.max_hp.saturating_sub(x).max(1);
    if p.hp > p.max_hp as i32 {
        p.hp = p.max_hp as i32;
    }
    set_flag(ctx.core, owner, "dcxiaoxi_x", x);
    if dcxiaoxi_in_range_targets(ctx.core, ctx.catalog, owner).is_empty() {
        set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
        set_flag(ctx.core, owner, "l27_b_ask_id", 0);
        return Flow::Done;
    }
    wait_ask_b(m, ctx, HostStage::TargetChoice, ASK_DCXIAOXI_TARGET)
}

fn dcxiaoxi_can_use_sha_on(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    t: PlayerId,
    check_usable: bool,
) -> bool {
    if !core.player(owner).alive || !core.player(t).alive || owner == t {
        return false;
    }
    let kongcheng = catalog
        .skill_by_name(crate::vocabulary::names::KONG_CHENG)
        .is_some_and(|d| core.has_skill(t, d));
    if kongcheng && core.player(t).hand.is_empty() {
        return false;
    }
    let tp = core.player(t);
    if tp.jidong_active || tp.yinshen_active || tp.jileng_active || tp.yinmi_active {
        return false;
    }
    if crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core,
        catalog,
        owner,
        t,
        0,
        CardKind::Sha,
        DamageNature::Normal,
    ) {
        return false;
    }
    if check_usable {
        let (limit, unlimited) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
            core,
            catalog,
            owner,
            None,
            CardKind::Sha,
            DamageNature::Normal,
            1,
        );
        if !unlimited && core.turn.sha_used_play_phase && limit <= 1 {
            return false;
        }
        let dist_unlimited = crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
            core,
            catalog,
            owner,
            0,
            CardKind::Sha,
            DamageNature::Normal,
        );
        if !dist_unlimited && core.computed_distance(catalog, owner, t) > core.attack_range(catalog, owner)
        {
            return false;
        }
    }
    true
}

fn dcxiaoxi_use_next_sha(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let rem = m.card.unwrap_or(0);
    let Some(t) = m.target else {
        set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
        return Flow::Done;
    };
    if rem == 0 || !dcxiaoxi_can_use_sha_on(ctx.core, ctx.catalog, owner, t, false) {
        set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
        return Flow::Done;
    }
    m.card = Some(rem - 1);
    m.stage = HostStage::DrawSettle;
    // 上游 `extension.js:6882`：`player.useCard({name:'sha',isCard:true}, target, false)` 属纯虚拟杀，牌堆为空时仍正常使用。
    ctx.emit(crate::event::EventData::CardUsed(crate::event::CardUseData {
        card: None,
        player: owner,
        targets: vec![t],
        nature: DamageNature::Normal,
        use_as: Some(CardKind::Sha),
        cancelled: false,
    }));
    Flow::Yield
}

/// `dcxiaoxi` 步进（`选将美化/extension.js:6774-6886`）：严格攻击范围目标、选牌获牌与循环真实使用虚拟杀。
pub(super) fn step_dcxiaoxi_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let max_hp = ctx.core.player(owner).max_hp;
            if max_hp <= 1 {
                return Flow::Done;
            }
            if max_hp == 2 {
                return dcxiaoxi_apply_lose_max_hp(m, ctx, owner, 1);
            }
            wait_ask_b(m, ctx, HostStage::SelectNature, ASK_DCXIAOXI_COUNT)
        }
        HostStage::SelectNature => {
            let max_lose = ctx.core.player(owner).max_hp.saturating_sub(1).min(2);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_lose.max(1)),
                _ => 1,
            };
            dcxiaoxi_apply_lose_max_hp(m, ctx, owner, x)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(t);
            let x = flag(ctx.core, owner, "dcxiaoxi_x").clamp(1, 2);
            let bool1 = !hand_and_equips(ctx.core, t).is_empty();
            let bool2 = dcxiaoxi_can_use_sha_on(ctx.core, ctx.catalog, owner, t, true);
            match (bool1, bool2) {
                (false, false) => {
                    set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
                    set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                    Flow::Done
                }
                (true, true) => wait_ask_b(m, ctx, HostStage::SelectCard, ASK_DCXIAOXI_MODE),
                (true, false) => wait_ask_b(m, ctx, HostStage::CardsAsk, ASK_DCXIAOXI_GAIN_CARD),
                (false, true) => {
                    set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
                    set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                    m.card = Some(x);
                    dcxiaoxi_use_next_sha(m, ctx, owner)
                }
            }
        }
        HostStage::SelectCard => {
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => n & 1,
                _ => 0,
            };
            let t = m.target.unwrap_or(owner);
            let x = flag(ctx.core, owner, "dcxiaoxi_x").clamp(1, 2);
            if mode == 0 {
                if !hand_and_equips(ctx.core, t).is_empty() {
                    return wait_ask_b(m, ctx, HostStage::CardsAsk, ASK_DCXIAOXI_GAIN_CARD);
                }
                set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            m.card = Some(x);
            dcxiaoxi_use_next_sha(m, ctx, owner)
        }
        HostStage::CardsAsk => {
            let t = m.target.unwrap_or(owner);
            let mut chosen = Vec::new();
            let hand_snapshot = ctx.core.player(t).hand.clone();
            for opt in ctx.answer().clone() {
                match opt {
                    ChoiceOption::HiddenCard(slot) => {
                        if let Some(&c) = hand_snapshot.get(slot as usize) {
                            if !chosen.contains(&c) {
                                chosen.push(c);
                            }
                        }
                    }
                    ChoiceOption::Card(c) => {
                        if !chosen.contains(&c) {
                            chosen.push(c);
                        }
                    }
                    _ => {}
                }
            }
            move_cards_to_hand(ctx, t, owner, &chosen);
            set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        HostStage::DrawSettle => dcxiaoxi_use_next_sha(m, ctx, owner),
        _ => Flow::Done,
    }
}

/// `twfengji` 步进（`选将美化/extension.js:6889-7043`）：`global: phaseBegin` 计回合，仅按 `card.name` 搜 `deck`，不足即停不补抽。
pub(super) fn step_twfengji_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) {
                resolve_twfengji_mahou(ctx, owner);
                return Flow::Done;
            }
            wait_ask_b(m, ctx, HostStage::SelectCard, ASK_TWFENGJI_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask_b(m, ctx, HostStage::SelectNature, ASK_TWFENGJI_TURNS)
        }
        HostStage::SelectNature => {
            let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_x),
                _ => 1,
            };
            if let Some(c) = m.card {
                ctx.take_owned_card(owner, c);
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "twfengji".to_string()))
                    .or_default()
                    .push(c);
                set_flag(
                    ctx.core,
                    owner,
                    "twfengji_kind",
                    (ctx.catalog.card_kind(c) as u32) + 1,
                );
            }
            finish_mahou_cast(m, ctx, "twfengji", x)
        }
        HostStage::DrawSettle => finish_renigong_draw(ctx, owner),
        _ => Flow::Done,
    }
}

fn fengji_card_name(cd: &crate::catalog::CardDef) -> &'static str {
    if cd.kind.is_sha_like() {
        "sha"
    } else {
        cd.name
    }
}

fn resolve_twfengji_mahou(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let rem = flag(ctx.core, owner, "twfengji_rem");
    if rem > 1 {
        set_flag(ctx.core, owner, "twfengji_rem", rem - 1);
        return;
    }
    set_flag(ctx.core, owner, "twfengji_rem", 0);
    let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
    let x = flag(ctx.core, owner, "twfengji_x").clamp(1, max_x) as usize;
    set_flag(ctx.core, owner, "twfengji_x", 0);
    set_flag(ctx.core, owner, "twfengji_kind", 0);
    let exp = ctx
        .core
        .vm_aux
        .piles
        .remove(&(owner, "twfengji".to_string()))
        .unwrap_or_default();
    if exp.is_empty() {
        return;
    }
    let exp_names: Vec<&'static str> = exp
        .iter()
        .filter_map(|&c| ctx.catalog.cards.get(c as usize).map(fengji_card_name))
        .collect();
    for &c in &exp {
        ctx.to_discard(c);
    }
    (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &exp, false);

    let mut gained = Vec::new();
    for target_name in exp_names {
        for _ in 0..x {
            if let Some(pos) = ctx.core.deck.iter().rposition(|&c| {
                ctx.catalog
                    .cards
                    .get(c as usize)
                    .is_some_and(|cd| fengji_card_name(cd) == target_name)
                    && !gained.contains(&c)
            }) {
                let c = ctx.core.deck.remove(pos);
                gained.push(c);
            } else {
                break;
            }
        }
    }
    if !gained.is_empty() {
        ctx.core.player_mut(owner).hand.extend(gained.iter().copied());
        ctx.log(LogEvent::CardsRevealed {
            cards: gained,
            to: Vec::new(),
        });
    }
}

/// `twzhouzu` 步进（`选将美化/extension.js:7441-7564`）：受害者自选弃手牌 `'h'`，然后受 1 点雷电伤害。
pub(super) fn step_twzhouzu_fix(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                let rem = flag(ctx.core, owner, "twzhouzu_rem");
                if rem > 1 {
                    set_flag(ctx.core, owner, "twzhouzu_rem", rem - 1);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "twzhouzu_rem", 0);
                let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
                let x = flag(ctx.core, owner, "twzhouzu_x").clamp(1, max_x);
                let t_enc = flag(ctx.core, owner, "twzhouzu_target");
                if t_enc == 0 {
                    set_flag(ctx.core, owner, "twzhouzu_x", 0);
                    return Flow::Done;
                }
                let t = (t_enc - 1) as PlayerId;
                if !ctx.core.player(t).alive {
                    set_flag(ctx.core, owner, "twzhouzu_x", 0);
                    set_flag(ctx.core, owner, "twzhouzu_target", 0);
                    return Flow::Done;
                }
                m.target = Some(t);
                m.card = Some(x);
                if !ctx.core.player(t).hand.is_empty() {
                    return wait_ask_b(m, ctx, HostStage::CardsAsk, ASK_TWZHOUZU_TARGET_DISCARD);
                }
                set_flag(ctx.core, owner, "twzhouzu_x", 0);
                set_flag(ctx.core, owner, "twzhouzu_target", 0);
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder);
                return Flow::Done;
            }
            wait_ask_b(m, ctx, HostStage::TargetChoice, ASK_TWZHOUZU_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask_b(m, ctx, HostStage::SelectNature, ASK_TWZHOUZU_TURNS)
        }
        HostStage::SelectNature => {
            let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_x),
                _ => 1,
            };
            if let Some(t) = m.target {
                set_flag(ctx.core, owner, "twzhouzu_target", u32::from(t) + 1);
            }
            finish_mahou_cast_stage(m, ctx, "twzhouzu", x, HostStage::SelectCard)
        }
        HostStage::SelectCard => finish_renigong_draw(ctx, owner),
        HostStage::CardsAsk => {
            let t = m.target.unwrap_or(owner);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) if ctx.core.player(t).hand.contains(&c) => Some(c),
                    _ => None,
                })
                .collect();
            set_flag(ctx.core, owner, "twzhouzu_x", 0);
            set_flag(ctx.core, owner, "twzhouzu_target", 0);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            if !chosen.is_empty() {
                discard_cards(ctx, t, &chosen);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            if ctx.core.player(t).alive {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder);
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            let t = m.target.unwrap_or(owner);
            if ctx.core.player(t).alive {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
