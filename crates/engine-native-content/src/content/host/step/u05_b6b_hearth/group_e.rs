//! Batch U05-6b (`shahai-hearth` Part 2) Group E: 15 技能 (`huanjue` .. `jihuo` + `stone_chenshui`)。

use super::asks::{
    ASK_B6B_BINGJIA_CARD, ASK_B6B_HUANJUE_PICK, ASK_B6B_JIHUO_DISCARD, ASK_B6B_JINGXIANG_CARD,
    ASK_B6B_JINGXIANG_TARGET, ASK_B6B_MDZHOUFU_CARD, ASK_B6B_MDZHOUFU_TARGET,
    ASK_B6B_MENGUN_DISCARD, ASK_B6B_WLIANJI_CONFIRM, ASK_B6B_XIANZHI_CHOICE, clear_b6b_ask,
    wait_b6b,
};
use super::{
    b6b_flag, can_jingxiang_card_respond, clear_b6b_flag, hs_b6b_dec, is_b6b_red_suit,
    pop_stashed_card, set_b6b_flag, spawn_random_dream_card, stash_card,
};
use super::super::u05_hearth::{
    alive_others, discard_card_from_zone, exact_base_name, gain_card_to_hand, grant_skill,
    has_any_skill, is_hearth_unique_equip, meta, nature_for_kind, remove_skill, rng_pick_mut,
    spawn_card_by_base,
};
use super::super::{HostMachine, HostSkillBehavior, HostStage};
use crate::catalog::{CardKind, EquipSlot, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_e(name: &str) -> Option<TriggerMeta> {
    match name {
        "huanjue" => Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())),
        "bingjia2" => Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())),
        "mianyi" => Some(meta(
            EventKind::DamageBefore,
            Timing::Normal,
            100,
            false,
            vec![EventKind::TurnPhaseAfter],
        )),
        "zuzhou" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd],
        )),
        "wlianji" | "stone_chenshui" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        "mengun" => Some(meta(EventKind::CardUsed, Timing::Normal, 12, false, Vec::new())),
        "jihuo" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, -50, false, Vec::new())),
        // `bianxing2` 空占位；`mdzhoufu2`/`xianzhi`/`jingxiang` 由内核挂点触发；`mengun2` 被动禁牌。
        _ => None,
    }
}

pub(super) fn limits_e(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn has_active_e(name: &str) -> bool {
    matches!(name, "bingjia" | "mdzhoufu" | "jianren")
}

fn suit_code(s: Suit) -> u32 {
    match s {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

fn is_huanjue_candidate_legal(
    state: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    user: PlayerId,
    tgt: PlayerId,
    cid: CardId,
) -> bool {
    if is_hearth_unique_equip(catalog, cid)
        || exact_base_name(catalog.cards[cid as usize].name) == "du"
    {
        return false;
    }
    let kind = catalog.card_kind(cid);
    if matches!(
        kind,
        CardKind::Wugu
            | CardKind::Taoyuan
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Jingleishan
            | CardKind::Chiyuxi
            | CardKind::Xianluhui
            | CardKind::Jiedao
            | CardKind::Shan
            | CardKind::Nullify
            | CardKind::Yuansuhuimie
            | CardKind::Dizaizhen
            | CardKind::Changshezhen
            | CardKind::Pozhenjue
            | CardKind::Tianfuzhen
            | CardKind::Qixingzhen
            | CardKind::Niaoxiangzhen
    ) {
        return false;
    }
    super::is_b6b_card_target_legal(
        state,
        catalog,
        user,
        tgt,
        Some(cid),
        Some(cid),
        kind,
        nature_for_kind(kind),
    )
}

fn huanjue_single_target(catalog: &crate::catalog::Catalog, u: &CardUseData) -> Option<PlayerId> {
    if u.targets.len() == 1 {
        return Some(u.targets[0]);
    }
    if u.targets.is_empty()
        && matches!(
            u.effective_kind(catalog),
            CardKind::Wuzhong
                | CardKind::Tao
                | CardKind::Hufu
                | CardKind::Jiu
                | CardKind::Tianxianjiu
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Dunpaigedang
                | CardKind::Chuansongmen
                | CardKind::Tanshezhiren
                | CardKind::Xingjiegoutong
                | CardKind::Jihuocard
                | CardKind::Yangpijuan
        )
    {
        return Some(u.player);
    }
    None
}

pub(super) fn filter_e(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    match skill.name.as_str() {
        "huanjue" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player == owner
                    && huanjue_single_target(ctx.catalog, u)
                        .is_some_and(|tgt| st.player(tgt).alive)
        ),
        "bingjia2" => match event {
            EventData::CardUsed(u)
                if !u.cancelled && u.player != owner && u.targets.contains(&owner) =>
            {
                let Some(stashed) = st
                    .vm_aux
                    .piles
                    .get(&(owner, "hs_bingjia".to_string()))
                    .and_then(|v| v.last().copied())
                else {
                    return false;
                };
                let want_suit = ctx.catalog.cards[stashed as usize].suit;
                u.card
                    .is_some_and(|c| ctx.catalog.effective_card_suit(st, u.player, c) == want_suit)
            }
            _ => false,
        },
        "mianyi" => match event {
            EventData::Damage(d) => d.target == owner && !d.prevented,
            EventData::TurnEndWindow { after: true, .. } => {
                b6b_flag(st, owner, "hs_bingjia_mianyi") > 0
            }
            _ => false,
        },
        "zuzhou" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && st
                        .player(owner)
                        .hand
                        .iter()
                        .any(|&c| ctx.catalog.effective_card_suit(st, owner, c) == Suit::Spade)
                    && !st.alive_players().any(|p| {
                        st.player(p)
                            .judge
                            .iter()
                            .any(|&jc| exact_base_name(ctx.catalog.cards[jc as usize].name) == "fulei")
                    })
            }
            EventData::Damage(d) => d
                .card
                .is_some_and(|c| exact_base_name(ctx.catalog.cards[c as usize].name) == "fulei"),
            _ => false,
        },
        "wlianji" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner
                    && (b6b_flag(st, owner, "hs_b6b_turn_card_used") as i32) > st.player(owner).hp
        ),
        "mengun" => match event {
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player != owner
                    && st.turn.current == u.player
                    && !has_any_skill(st, ctx.catalog, u.player, "mengun2") =>
            {
                let Some(c) = u.card else {
                    return false;
                };
                if !ctx.catalog.card_kind(c).is_basic_kind() {
                    return false;
                }
                let suit = ctx.catalog.effective_card_suit(st, u.player, c);
                st.player(owner)
                    .hand
                    .iter()
                    .any(|&hc| ctx.catalog.effective_card_suit(st, owner, hc) == suit)
            }
            _ => false,
        },
        "jihuo" => matches!(
            event,
            EventData::TurnEndWindow { player, after: true }
                if *player == owner
                    && !st.player(owner).hand.is_empty()
                    && b6b_flag(st, owner, "hs_active_extra_turn_source") != 2
        ),
        "stone_chenshui" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner
        ),
        _ => false,
    }
}

pub(super) fn activate_e(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let st = ctx.state;
    match skill.name.as_str() {
        "bingjia" => Some(
            !has_any_skill(st, ctx.catalog, owner, "bingjia2")
                && !st.player(owner).hand.is_empty(),
        ),
        "mdzhoufu" => Some(
            st.player(owner)
                .hand
                .iter()
                .any(|&c| !is_b6b_red_suit(ctx.catalog.effective_card_suit(st, owner, c)))
                && alive_others(st, owner)
                    .into_iter()
                    .any(|p| !has_any_skill(st, ctx.catalog, p, "mdzhoufu2")),
        ),
        "jianren" => Some(
            super::hs_b6b_cnt(st, owner, "jianren") == 0
                && st
                    .player(owner)
                    .equips
                    .iter()
                    .any(|&c| ctx.catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon))
                && !alive_others(st, owner).is_empty(),
        ),
        _ => None,
    }
}

pub(super) fn step_e(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "huanjue" => match m.stage {
            HostStage::Act => {
                let fid = ctx.current_event_frame_id();
                if b6b_flag(ctx.core, m.owner, &format!("hs_huanjue_frame:{fid}")) > 0 {
                    return Flow::Done;
                }
                let Ok(u) = ctx.current_card_use() else {
                    return Flow::Done;
                };
                if u.cancelled {
                    return Flow::Done;
                }
                let Some(tgt) = huanjue_single_target(ctx.catalog, u) else {
                    return Flow::Done;
                };
                let user = u.player;
                let cat = ctx.catalog;
                let mut list1: Vec<CardId> = Vec::new();
                let mut list2: Vec<CardId> = Vec::new();
                for &(c, _) in &ctx.core.detached_cards {
                    if is_huanjue_candidate_legal(ctx.core, cat, user, tgt, c) {
                        list1.push(c);
                        if cat.card_kind(c).equip_slot().is_none() {
                            list2.push(c);
                        }
                    }
                }
                let mut picks = Vec::new();
                let mut has_equip = false;
                for _ in 0..3 {
                    let pool = if has_equip && !list2.is_empty() {
                        &mut list2
                    } else {
                        &mut list1
                    };
                    if let Some(c) = rng_pick_mut(ctx, pool) {
                        if ctx.catalog.card_kind(c).equip_slot().is_some() {
                            has_equip = true;
                        }
                        list1.retain(|&x| x != c);
                        list2.retain(|&x| x != c);
                        picks.push(c);
                    }
                }
                if picks.is_empty() {
                    return Flow::Done;
                }
                m.cards = picks;
                wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_HUANJUE_PICK)
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if ctx.current_card_use().is_err() {
                    return Flow::Done;
                }
                let fid = ctx.current_event_frame_id();
                set_b6b_flag(ctx.core, m.owner, &format!("hs_huanjue_frame:{fid}"), 1);
                ctx.core.detached_cards.retain(|&(x, _)| x != picked);
                let Ok(u_ro) = ctx.current_card_use() else {
                    return Flow::Done;
                };
                let (orig_card, user) = (u_ro.card, u_ro.player);
                if let Some(orig) = orig_card {
                    if ctx.core.player(user).hand.contains(&orig) {
                        ctx.take_hand_card(user, orig);
                    }
                    ctx.core.table.retain(|&x| x != orig);
                    ctx.to_discard(orig);
                }
                // `cardsGotoOrdering(card).relatedEvent = trigger`（skill.js:6396）：入处理区而非手牌；
                // 处于 BeforeTriggers 窗内改写，随后的 commit_card_used 将按新牌提交计数与效果。
                ctx.core.table.push(picked);
                let new_kind = ctx.catalog.card_kind(picked);
                let need_tgt = super::super::u05_hearth::u05_hearth_a_tail::hearth_card_needs_target(
                    ctx.core,
                    ctx.catalog,
                    user,
                    picked,
                );
                let u = ctx.current_card_use_mut();
                u.card = Some(picked);
                u.use_as = None;
                u.nature = nature_for_kind(new_kind);
                if u.targets.is_empty() && need_tgt {
                    u.targets = vec![user];
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bingjia" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_BINGJIA_CARD),
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    if ctx.core.player(m.owner).hand.contains(&cid) {
                        ctx.take_hand_card(m.owner, cid);
                        stash_card(ctx.core, m.owner, "hs_bingjia", cid);
                        grant_skill(ctx, m.owner, "bingjia2", false);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bingjia2" => {
            if let Some(stashed) = pop_stashed_card(ctx.core, m.owner, "hs_bingjia") {
                ctx.to_discard(stashed);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: vec![stashed],
                });
            }
            remove_skill(ctx, m.owner, "bingjia2");
            ctx.change_hujia(m.owner, 1);
            set_b6b_flag(ctx.core, m.owner, "hs_bingjia_mianyi", 1);
            grant_skill(ctx, m.owner, "mianyi", false);
            Flow::Done
        }
        "mianyi" => {
            if matches!(
                ctx.res.stack.last().map(|f| &f.data),
                Some(EventData::TurnEndWindow { after: true, .. })
            ) {
                if b6b_flag(ctx.core, m.owner, "hs_bingjia_mianyi") > 0 {
                    clear_b6b_flag(ctx.core, m.owner, "hs_bingjia_mianyi");
                    remove_skill(ctx, m.owner, "mianyi");
                }
            } else if ctx
                .current_damage()
                .is_ok_and(|d| d.target == m.owner && !d.prevented)
            {
                ctx.current_damage_mut().prevented = true;
            }
            Flow::Done
        }
        "bianxing2" => Flow::Done,
        "zuzhou" => {
            if let Ok(d) = ctx.current_damage() {
                if let Some(c) = d.card {
                    if exact_base_name(ctx.catalog.cards[c as usize].name) == "fulei" {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((0, format!("hs_fulei_expired:{c}")), 1);
                    }
                }
                return Flow::Done;
            }
            let any_fulei = ctx.core.alive_players().any(|p| {
                ctx.core
                    .player(p)
                    .judge
                    .iter()
                    .any(|&jc| exact_base_name(ctx.catalog.cards[jc as usize].name) == "fulei")
            });
            if !any_fulei {
                if let Some(fulei) = spawn_card_by_base(ctx, "fulei") {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(0, format!("hs_fulei_hits:{fulei}")));
                    ctx.core.player_mut(m.owner).judge.push(fulei);
                }
            }
            Flow::Done
        }
        "mdzhoufu" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_MDZHOUFU_CARD),
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_MDZHOUFU_TARGET)
            }
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let Some(cid) = m.card else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive
                        && t != m.owner
                        && !has_any_skill(ctx.core, ctx.catalog, t, "mdzhoufu2")
                        && ctx.core.player(m.owner).hand.contains(&cid)
                    {
                        ctx.take_hand_card(m.owner, cid);
                        stash_card(ctx.core, t, "hs_mdzhoufu2", cid);
                        set_b6b_flag(ctx.core, t, "hs_mdzhoufu3", u32::from(m.owner) + 1);
                        grant_skill(ctx, t, "mdzhoufu2", false);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xianzhi" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_XIANZHI_CHOICE),
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    && ctx.core.deck.len() > 1
                {
                    let len = ctx.core.deck.len();
                    ctx.core.deck.swap(len - 1, len - 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jingxiang" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_JINGXIANG_TARGET),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    hs_b6b_dec(ctx.core, m.owner, "jingxiang");
                    return Flow::Done;
                };
                let resp_ctx = m.cards.first().copied().unwrap_or(1) as u8;
                let has_any = ctx.core.player(t).alive
                    && ctx.core.player(t).hand.iter().any(|&c| {
                        can_jingxiang_card_respond(ctx.core, ctx.catalog, m.owner, c, resp_ctx)
                    });
                if !has_any {
                    hs_b6b_dec(ctx.core, m.owner, "jingxiang");
                    return Flow::Done;
                }
                m.target = Some(t);
                wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_JINGXIANG_CARD)
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    hs_b6b_dec(ctx.core, m.owner, "jingxiang");
                    return Flow::Done;
                };
                let Some(t) = m.target else {
                    hs_b6b_dec(ctx.core, m.owner, "jingxiang");
                    return Flow::Done;
                };
                let resp_ctx = m.cards.first().copied().unwrap_or(1) as u8;
                if !ctx.core.player(t).hand.contains(&cid)
                    || !can_jingxiang_card_respond(ctx.core, ctx.catalog, m.owner, cid, resp_ctx)
                {
                    hs_b6b_dec(ctx.core, m.owner, "jingxiang");
                    return Flow::Done;
                }
                ctx.take_hand_card(t, cid);
                ctx.note_cards_lost(t, &[cid]);
                ctx.emit(EventData::CardsLost {
                    player: t,
                    cards: vec![cid],
                    origins: vec![LostOrigin::Hand],
                });
                // 暂时挂入手牌通道，确保 kernel 五处响应消费/出牌管道不触发多余离手并守恒入弃牌堆。
                set_b6b_flag(ctx.core, m.owner, "hs_jingxiang_temp_card", cid + 1);
                ctx.core.player_mut(m.owner).hand.push(cid);
                set_b6b_flag(ctx.core, m.owner, "l10_reguhuo_resp_done", 1);
                set_b6b_flag(ctx.core, m.owner, "l10_reguhuo_resp_card", cid);
                if resp_ctx == 4 || resp_ctx == 5 {
                    let eff_k = super::super::u05_hearth_a_tail::effective_hearth_card(
                        ctx.core, ctx.catalog, m.owner, cid,
                    )
                    .1;
                    let rk = if eff_k == CardKind::Jiu { 5 } else { 4 };
                    set_b6b_flag(ctx.core, m.owner, "l10_reguhuo_resp_kind", rk);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "wlianji" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_WLIANJI_CONFIRM),
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    ctx.start_draw(m.owner, 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mengun" => match m.stage {
            HostStage::Act => {
                let Ok(u) = ctx.current_card_use() else {
                    return Flow::Done;
                };
                let Some(orig_c) = u.card else {
                    return Flow::Done;
                };
                let suit = ctx.catalog.effective_card_suit(ctx.core, u.player, orig_c);
                m.card = Some(suit_code(suit));
                wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_MENGUN_DISCARD)
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                let Some(ChoiceOption::Card(dis_cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(m.owner).hand.contains(&dis_cid)
                    || ctx.current_card_use().is_err()
                {
                    return Flow::Done;
                }
                let (orig_c, user) = {
                    let u = ctx.current_card_use_mut();
                    u.cancelled = true;
                    (u.card, u.player)
                };
                ctx.discard_from_hand(m.owner, dis_cid);
                if let Some(c) = orig_c {
                    ctx.core.table.retain(|&x| x != c);
                    if !ctx.core.player(user).hand.contains(&c) {
                        gain_card_to_hand(ctx, user, c);
                    }
                    set_b6b_flag(ctx.core, user, "hs_mengun2_card", c + 1);
                    grant_skill(ctx, user, "mengun2", true);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jianren" => {
            let weapon = ctx
                .core
                .player(m.owner)
                .equips
                .iter()
                .copied()
                .find(|&c| ctx.catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon));
            if let Some(w) = weapon {
                super::hs_b6b_inc(ctx.core, m.owner, "jianren");
                discard_card_from_zone(ctx, m.owner, w);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: vec![w],
                });
                let targets = alive_others(ctx.core, m.owner);
                for t in targets.into_iter().rev() {
                    ctx.emit(EventData::Damage(super::super::u05_hearth::make_damage(
                        Some(m.owner),
                        t,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    )));
                }
            }
            Flow::Done
        }
        "jihuo" => match m.stage {
            HostStage::Act => {
                if b6b_flag(ctx.core, m.owner, "hs_active_extra_turn_source") == 2 {
                    return Flow::Done;
                }
                wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_JIHUO_DISCARD)
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    if ctx.core.player(m.owner).hand.contains(&cid) {
                        ctx.discard_from_hand(m.owner, cid);
                        set_b6b_flag(ctx.core, m.owner, "extra_turn_insert_queued", 1);
                        set_b6b_flag(ctx.core, m.owner, "hs_queued_extra_turn_source", 2);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "stone_chenshui" => {
            let leader_raw = b6b_flag(ctx.core, m.owner, "u05_stone_leader");
            let target = if leader_raw > 0 {
                let p = (leader_raw - 1) as PlayerId;
                if (p as usize) < ctx.core.players.len() && ctx.core.player(p).alive {
                    p
                } else {
                    m.owner
                }
            } else {
                m.owner
            };
            spawn_random_dream_card(ctx, target);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
