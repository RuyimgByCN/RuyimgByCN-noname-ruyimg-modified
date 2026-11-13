//! L25 祖安武将 Batch E 后半技能步进（技能 13–22：`ssr_nianlei` 至 `ssr_newzhiba`）。

use super::l25_batch_e_asks::{discard_from_hej, is_confirmed, refund_once_used, wait_ask};
use super::*;
use crate::catalog::{CardKind, Faction, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_l25_batch_e_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "ssr_jiqiao" => step_ssr_jiqiao(m, ctx),
        "ssr_qicai" => step_ssr_qicai(m, ctx),
        "ssr_newweimu" => step_ssr_newweimu(m, ctx),
        "ssr_guimou" => step_ssr_guimou(m, ctx),
        "ssr_newluanwu" => step_ssr_newluanwu(m, ctx),
        "ssr_nianlei" => step_ssr_nianlei(m, ctx),
        "ssr_newduanxian" => step_ssr_newduanxian(m, ctx),
        "ssr_manjia" => step_ssr_manjia(m, ctx),
        "ssr_zaiqi" => step_ssr_zaiqi(m, ctx),
        "ssr_lieren" => step_ssr_lieren(m, ctx),
        "ssr_zhiji" => step_ssr_zhiji(m, ctx),
        "ssr_weilong" => step_ssr_weilong(m, ctx),
        "ssr_hunzi" => step_ssr_hunzi(m, ctx),
        "ssr_newjiang" => step_ssr_newjiang(m, ctx),
        "ssr_newzhiba" => step_ssr_newzhiba(m, ctx),
        _ => Flow::Done,
    }
}

fn top_up_jiqiao_pile(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut pile = jiqiao_pile(ctx.core, owner);
    while pile.len() < 3 {
        if let Some(cid) = ctx.core.deck.pop() {
            pile.push(cid);
        } else if let Some(cid) = ctx.core.discard.pop() {
            pile.push(cid);
        } else {
            break;
        }
    }
    let len = pile.len() as u32;
    if !pile.is_empty() {
        ctx.core
            .vm_aux
            .piles
            .insert((owner, "ssr_jiqiao".to_string()), pile);
    }
    set_flag(ctx.core, owner, "l25_jiqiao_count", len);
    set_flag(ctx.core, owner, "mark:ssr_jiqiao", len);
}

fn swap_hand_and_jiqiao(ctx: &mut Ctx<'_>, owner: PlayerId, h_cid: CardId, pile_idx: usize) {
    let Some(h_pos) = ctx.core.player(owner).hand.iter().position(|&x| x == h_cid) else {
        return;
    };
    let mut pile = jiqiao_pile(ctx.core, owner);
    if pile.is_empty() {
        return;
    }
    let idx = pile_idx.min(pile.len() - 1);
    let p_cid = pile[idx];
    pile[idx] = h_cid;
    ctx.core.player_mut(owner).hand[h_pos] = p_cid;
    let len = pile.len() as u32;
    ctx.core
        .vm_aux
        .piles
        .insert((owner, "ssr_jiqiao".to_string()), pile);
    set_flag(ctx.core, owner, "l25_jiqiao_count", len);
    set_flag(ctx.core, owner, "mark:ssr_jiqiao", len);
    set_flag(ctx.core, owner, "l25_jiqiao_swapped", 1);
}

fn step_ssr_jiqiao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(108) | Some(110) | Some(114)) {
        top_up_jiqiao_pile(ctx, owner);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            if jiqiao_pile(ctx.core, owner).is_empty() {
                top_up_jiqiao_pile(ctx, owner);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jiqiao");
            set_flag(ctx.core, owner, "l25_jiqiao_used", cnt);
            ctx.start_draw(owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).hand.is_empty() || jiqiao_pile(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_JIQIAO_HAND)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(h_cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![h_cid];
            if jiqiao_pile(ctx.core, owner).len() <= 1 {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                swap_hand_and_jiqiao(ctx, owner, h_cid, 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_JIQIAO_PILE)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => usize::from(v),
                _ => 0,
            };
            if let Some(&h_cid) = m.cards.first() {
                swap_hand_and_jiqiao(ctx, owner, h_cid, idx);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_qicai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l25_qicai_triggered", 1);
            if alive_others(ctx.core, owner).is_empty() {
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_QICAI_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, "l25_qicai_extra_target", u32::from(tgt) + 1);
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn flip_player_face(ctx: &mut Ctx<'_>, p: PlayerId) -> bool {
    let flipped = {
        let pl = ctx.core.player_mut(p);
        pl.face_down = !pl.face_down;
        pl.face_down
    };
    ctx.emit(EventData::TurnOvered {
        player: p,
        face_down: flipped,
    });
    flipped
}

fn step_ssr_newweimu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(102) | Some(110) | Some(113)) {
        let active = if ctx.core.player(owner).face_down {
            1
        } else {
            0
        };
        set_flag(ctx.core, owner, "l25_weimu_active", active);
        if m.card == Some(102) && m.target != Some(owner) && active > 0 {
            set_flag(ctx.core, owner, "l25_weimu_nullified", 1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWWEIMU_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWWEIMU_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if let Some(&cid) = m.cards.first() {
                discard_he_cards(ctx, owner, &[cid]);
            }
            set_flag(ctx.core, owner, "l25_weimu_active", 1);
            set_flag(ctx.core, owner, "l25_weimu_dismantle", 1);
            if let Some(&victim_cid) = hej_cards(ctx.core, tgt).first() {
                discard_from_hej(ctx, tgt, victim_cid);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_guimou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_GUIMOU_CONFIRM),
        HostStage::SelectNature => {
            if !is_confirmed(ctx) {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            }
            let flipped = flip_player_face(ctx, owner);
            let active = if flipped { 1 } else { 0 };
            set_flag(ctx.core, owner, "l25_weimu_active", active);
            set_flag(ctx.core, owner, "l25_guimou_triggered", 1);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_GUIMOU_TARGETS)
        }
        HostStage::TargetChoice => {
            let chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if chosen.len() < 2 {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            }
            let p1 = chosen[0];
            let p2 = chosen[1];
            m.targets = vec![p1, p2];
            m.target = Some(p1);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_GUIMOU_PINDIAN_1)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![c1];
            let p2 = m.targets.get(1).copied().unwrap_or(owner);
            m.target = Some(p2);
            wait_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_GUIMOU_PINDIAN_2)
        }
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let p1 = m.targets.first().copied().unwrap_or(owner);
            let p2 = m.targets.get(1).copied().unwrap_or(owner);
            let c1 = m.cards.first().copied().unwrap_or(c2);
            let r1 = card_rank(ctx.catalog, c1);
            let r2 = card_rank(ctx.catalog, c2);
            if ctx.core.player(p1).hand.contains(&c1) {
                ctx.discard_from_hand(p1, c1);
            }
            if ctx.core.player(p2).hand.contains(&c2) {
                ctx.discard_from_hand(p2, c2);
            }
            if r1 != r2 {
                let (winner, loser) = if r1 > r2 { (p1, p2) } else { (p2, p1) };
                set_flag(ctx.core, winner, "l25_guimou_win", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(winner),
                    loser,
                    1,
                    DamageNature::Normal,
                    None,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newluanwu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWLUANWU_CONFIRM),
        HostStage::SelectNature => {
            if !is_confirmed(ctx) {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                refund_once_used(ctx, owner, "ssr_newluanwu");
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l25_newluanwu_used", 1);
            m.targets = alive_all(ctx.core, owner);
            let Some(&first) = m.targets.first() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(first);
            wait_ask(m, ctx, HostStage::JiefenTarget, ASK_NEWLUANWU_MODE)
        }
        HostStage::JiefenTarget => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            let cur_p = m.target.unwrap_or(owner);
            m.targets.retain(|&p| p != cur_p);
            let flipped = flip_player_face(ctx, cur_p);
            if cur_p == owner {
                let active = if flipped { 1 } else { 0 };
                set_flag(ctx.core, owner, "l25_weimu_active", active);
            }
            if mode == 0 {
                ctx.start_draw(cur_p, 2);
            } else {
                let zd = flag(ctx.core, cur_p, "l25_luanwu_zhendu") + 1;
                set_flag(ctx.core, cur_p, "l25_luanwu_zhendu", zd);
                ctx.emit(EventData::LoseHp {
                    player: cur_p,
                    amount: 1,
                });
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            m.targets.retain(|&p| ctx.core.player(p).alive);
            if let Some(&next_p) = m.targets.first() {
                m.target = Some(next_p);
                return wait_ask(m, ctx, HostStage::JiefenTarget, ASK_NEWLUANWU_MODE);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_nianlei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(110) | Some(114)) {
        if flag(ctx.core, owner, "l25_nianlei_init") == 0 {
            set_flag(ctx.core, owner, "l25_nianlei_init", 1);
            set_flag(ctx.core, owner, "l25_nianlei_chenqing", 1);
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    if m.card == Some(102) {
        let atk = m.target.unwrap_or(owner);
        set_flag(ctx.core, atk, "l25_nianlei_chenqing", 0);
        set_flag(ctx.core, atk, "l25_nianlei_shangai", 1);
        set_flag(ctx.core, owner, "l25_nianlei_triggered", 1);
        if let Some(&cid) = he_cards(ctx.core, atk).first() {
            discard_he_cards(ctx, atk, &[cid]);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NIANLEI_GIVE),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NIANLEI_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_nianlei");
            set_flag(ctx.core, owner, "l25_nianlei_used", cnt);
            if let Some(&cid) = m.cards.first() {
                if ctx.core.player(owner).hand.contains(&cid) {
                    ctx.give_hand_card(owner, tgt, cid);
                }
            }
            set_flag(ctx.core, tgt, "l25_nianlei_shangai", 0);
            set_flag(ctx.core, tgt, "l25_nianlei_chenqing", 1);
            ctx.start_draw(tgt, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newduanxian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWDUANXIAN_DISCARD),
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if chosen.len() < 2 {
                return Flow::Done;
            }
            discard_he_cards(ctx, owner, &chosen[..2]);
            set_flag(ctx.core, owner, "l25_newduanxian_used", 1);
            let mut yielded = false;
            for p in alive_all(ctx.core, owner) {
                if flag(ctx.core, p, "l25_nianlei_chenqing") > 0 {
                    ctx.recover(p, 2, None);
                    ctx.start_draw(p, 1);
                    yielded = true;
                }
                if flag(ctx.core, p, "l25_nianlei_shangai") > 0 {
                    ctx.emit(EventData::LoseHp {
                        player: p,
                        amount: 2,
                    });
                    yielded = true;
                }
            }
            if yielded {
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_manjia(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage == HostStage::Act {
        set_flag(ctx.core, m.owner, "l25_manjia_triggered", 1);
    }
    Flow::Done
}

fn step_ssr_zaiqi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(110) | Some(114)) {
        if flag(ctx.core, owner, "l25_zaiqi_init") == 0 {
            set_zaiqi_marks(ctx.core, owner, 7);
        }
        return Flow::Done;
    }
    if m.card == Some(105) {
        let cur = zaiqi_marks(ctx.core, owner);
        if cur > 0 {
            set_zaiqi_marks(ctx.core, owner, cur - 1);
            let need = (1 - ctx.core.player(owner).hp).max(1) as u32;
            ctx.recover(owner, need, None);
            set_flag(ctx.core, owner, "l25_zaiqi_dying_saved", 1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            if m.card == Some(101) {
                return wait_ask(m, ctx, HostStage::SelectNature, ASK_ZAIQI_CONFIRM);
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_ZAIQI_DISCARD)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            if is_confirmed(ctx) {
                let cur = zaiqi_marks(ctx.core, owner);
                if cur > 0 {
                    set_zaiqi_marks(ctx.core, owner, cur - 1);
                    ctx.core.player_mut(owner).linked = true;
                    ctx.recover(owner, 1, None);
                    set_flag(ctx.core, owner, "l25_zaiqi_triggered", 1);
                }
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            let cur = zaiqi_marks(ctx.core, owner);
            if chosen.len() < 2 || cur == 0 {
                return Flow::Done;
            }
            set_zaiqi_marks(ctx.core, owner, cur - 1);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_zaiqi");
            set_flag(ctx.core, owner, "l25_zaiqi_used", cnt);
            discard_he_cards(ctx, owner, &chosen[..2]);
            let others = alive_others(ctx.core, owner);
            if !others.is_empty() {
                if !ctx.core.deck.is_empty() {
                    ctx.use_virtual_card(owner, CardKind::SavageAttack, others);
                } else {
                    for p in others {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(owner),
                            p,
                            1,
                            DamageNature::Normal,
                            None,
                        )));
                    }
                }
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn reclaim_all_lieren(ctx: &mut Ctx<'_>, owner: PlayerId, holder: PlayerId) -> bool {
    let key = (holder, "ssr_lieren".to_string());
    let mut reclaimed = Vec::new();
    if let Some(pile) = ctx.core.vm_aux.piles.remove(&key) {
        for cid in pile {
            ctx.core.player_mut(owner).hand.push(cid);
            reclaimed.push(cid);
        }
    }
    let had_flag = flag(ctx.core, holder, "l25_lieren_en") > 0;
    set_flag(ctx.core, holder, "l25_lieren_en", 0);
    if !reclaimed.is_empty() {
        ctx.out.push(LogEvent::CardsDrawn {
            player: owner,
            cards: reclaimed,
        });
        set_flag(ctx.core, owner, "l25_lieren_reclaimed", 1);
        true
    } else if had_flag {
        set_flag(ctx.core, owner, "l25_lieren_reclaimed", 1);
        true
    } else {
        false
    }
}

fn advance_lieren_targets(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    while let Some(&tgt) = m.targets.first() {
        if !ctx.core.player(tgt).alive {
            m.targets.remove(0);
            continue;
        }
        let has_shan = ctx
            .core
            .player(tgt)
            .hand
            .iter()
            .any(|&c| ctx.catalog.card_kind(c) == CardKind::Shan);
        if has_shan {
            m.target = Some(tgt);
            return wait_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_LIEREN_RESPONSE);
        }
        m.targets.remove(0);
        set_flag(ctx.core, owner, "l25_lieren_damaged", 1);
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            tgt,
            1,
            DamageNature::Normal,
            None,
        )));
        m.stage = HostStage::DrawSettle;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_ssr_lieren(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(110) | Some(114)) {
        if flag(ctx.core, owner, "l25_lieren_init") == 0 {
            set_flag(ctx.core, owner, "l25_lieren_init", 1);
            let fetched = fetch_card_matching(ctx, |c, cat| is_weapon_card(cat, c))
                .or_else(|| fetch_card_matching(ctx, |c, cat| cat.card_kind(c).is_equip()));
            if let Some(cid) = fetched {
                ctx.core.player_mut(owner).hand.push(cid);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![cid],
                });
                set_flag(ctx.core, owner, "l25_lieren_fetched", cid + 1);
            }
        }
        return Flow::Done;
    }
    if m.card == Some(112) {
        if let Some(dead_p) = m.target {
            reclaim_all_lieren(ctx, owner, dead_p);
        }
        return Flow::Done;
    }
    if matches!(m.card, Some(108) | Some(105)) && m.stage == HostStage::Act {
        let mut affected = Vec::new();
        for p in alive_others(ctx.core, owner) {
            if reclaim_all_lieren(ctx, owner, p) {
                affected.push(p);
            }
        }
        m.targets = affected;
        return advance_lieren_targets(m, ctx);
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_LIEREN_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_LIEREN_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_lieren");
            set_flag(ctx.core, owner, "l25_lieren_used", cnt);
            if let Some(&cid) = m.cards.first() {
                if ctx.core.player(owner).hand.contains(&cid)
                    || ctx.core.player(owner).equips.contains(&cid)
                {
                    ctx.take_owned_card(owner, cid);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((tgt, "ssr_lieren".to_string()))
                        .or_default()
                        .push(cid);
                }
            }
            set_flag(ctx.core, tgt, "l25_lieren_en", 1);
            Flow::Done
        }
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            m.targets.retain(|&p| p != tgt);
            if let Some(ChoiceOption::Card(shan_cid)) = ctx.answer().first().copied() {
                if ctx.core.player(tgt).hand.contains(&shan_cid) {
                    ctx.discard_from_hand(tgt, shan_cid);
                }
                return advance_lieren_targets(m, ctx);
            }
            set_flag(ctx.core, owner, "l25_lieren_damaged", 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Normal,
                None,
            )));
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => advance_lieren_targets(m, ctx),
        _ => Flow::Done,
    }
}

fn step_ssr_zhiji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "l25_zhiji_stage") >= 3 {
                return Flow::Done;
            }
            if m.card == Some(105) {
                let dp = m.target.unwrap_or(owner);
                let mask = flag(ctx.core, owner, "l25_zhiji_dying_mask") | (1u32 << (dp as u32));
                set_flag(ctx.core, owner, "l25_zhiji_dying_mask", mask);
            }
            let new_max = ctx.core.player(owner).max_hp.saturating_sub(1).max(1);
            ctx.core.player_mut(owner).max_hp = new_max;
            if ctx.core.player(owner).hp > new_max as i32 {
                ctx.core.player_mut(owner).hp = new_max as i32;
            }
            let hj = (ctx.core.player(owner).hujia + 1).min(5);
            ctx.core.player_mut(owner).hujia = hj;
            let stg = flag(ctx.core, owner, "l25_zhiji_stage") + 1;
            set_flag(ctx.core, owner, "l25_zhiji_stage", stg);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_ZHIJI_OPTION)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let opt = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            match opt {
                0 => set_flag(ctx.core, owner, "l25_zhiji_zhengnan", 1),
                1 => set_flag(ctx.core, owner, "l25_zhiji_fabei", 1),
                _ => set_flag(ctx.core, owner, "l25_zhiji_wanlan", 1),
            }
            ctx.start_draw(owner, 2);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_weilong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_WEILONG_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            let suit = ctx.catalog.effective_card_suit(ctx.core, owner, cid);
            if suit == Suit::Diamond && !alive_others(ctx.core, owner).is_empty() {
                m.cards = vec![cid];
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_WEILONG_TARGET);
            }
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_weilong");
            set_flag(ctx.core, owner, "l25_weilong_used", cnt);
            discard_he_cards(ctx, owner, &[cid]);
            match suit {
                Suit::Heart => {
                    ctx.recover(owner, 1, None);
                    ctx.start_draw(owner, 1);
                }
                Suit::Club => {
                    set_flag(ctx.core, owner, "l25_weilong_shan_guard", 1);
                    ctx.start_draw(owner, 2);
                }
                Suit::Spade => {
                    ctx.core.turn.jiu_active = true;
                    set_flag(ctx.core, owner, "l25_weilong_jiu", 1);
                    ctx.start_draw(owner, 1);
                }
                Suit::Diamond => {
                    ctx.start_draw(owner, 1);
                }
            }
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_weilong");
            set_flag(ctx.core, owner, "l25_weilong_used", cnt);
            if let Some(&cid) = m.cards.first() {
                discard_he_cards(ctx, owner, &[cid]);
            }
            if !ctx.core.deck.is_empty() {
                ctx.use_virtual_card(owner, CardKind::FireSha, vec![tgt]);
            } else {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_hunzi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_HUNZI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_HUNZI_PINDIAN)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![c1];
            wait_ask(
                m,
                ctx,
                HostStage::JiefenTargetDiscard,
                ASK_HUNZI_TARGET_PINDIAN,
            )
        }
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let tgt = m.target.unwrap_or(owner);
            let c1 = m.cards.first().copied().unwrap_or(c2);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_hunzi");
            set_flag(ctx.core, owner, "l25_hunzi_used", cnt);
            let boost = flag(ctx.core, owner, "l25_zhiba_boost") as i32;
            let penalty = flag(ctx.core, tgt, "l25_hunzi_xin") as i32;
            let r1 = (card_rank(ctx.catalog, c1) as i32 + boost).clamp(1, 13);
            let r2 = (card_rank(ctx.catalog, c2) as i32 - penalty).clamp(1, 13);
            if ctx.core.player(owner).hand.contains(&c1) {
                ctx.discard_from_hand(owner, c1);
            }
            if ctx.core.player(tgt).hand.contains(&c2) {
                ctx.discard_from_hand(tgt, c2);
            }
            if r1 > r2 {
                let xin = flag(ctx.core, tgt, "l25_hunzi_xin") + 1;
                set_flag(ctx.core, tgt, "l25_hunzi_xin", xin);
                let tot = flag(ctx.core, owner, "l25_hunzi_total_xin") + 1;
                set_flag(ctx.core, owner, "l25_hunzi_total_xin", tot);
                let mut draw_n = 1u32;
                if (xin >= 3 || tot >= 3) && flag(ctx.core, owner, "l25_hunzi_awakened") == 0 {
                    set_flag(ctx.core, owner, "l25_hunzi_awakened", 1);
                    let new_max = ctx.core.player(owner).max_hp.saturating_sub(1).max(1);
                    ctx.core.player_mut(owner).max_hp = new_max;
                    if ctx.core.player(owner).hp > new_max as i32 {
                        ctx.core.player_mut(owner).hp = new_max as i32;
                    }
                    let hj = (ctx.core.player(owner).hujia + 1).min(5);
                    ctx.core.player_mut(owner).hujia = hj;
                    draw_n += 2;
                }
                ctx.start_draw(owner, draw_n);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newjiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    set_flag(ctx.core, m.owner, "l25_newjiang_triggered", 1);
    ctx.start_draw(m.owner, 1);
    m.stage = HostStage::CardsAsk;
    Flow::Yield
}

fn step_ssr_newzhiba(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let resp = alive_others(ctx.core, owner)
                .into_iter()
                .find(|&p| {
                    ctx.core.effective_faction(ctx.catalog, p) == Some(Faction::Wu)
                        && !he_cards(ctx.core, p).is_empty()
                })
                .or_else(|| {
                    alive_others(ctx.core, owner)
                        .into_iter()
                        .find(|&p| !he_cards(ctx.core, p).is_empty())
                });
            let Some(resp_p) = resp else {
                return Flow::Done;
            };
            m.target = Some(resp_p);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWZHIBA_DISCARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_e_ask_id", 0);
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let resp = m.target.unwrap_or(owner);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newzhiba");
            set_flag(ctx.core, owner, "l25_newzhiba_used", cnt);
            discard_he_cards(ctx, resp, &[cid]);
            let boost = flag(ctx.core, owner, "l25_zhiba_boost") + 1;
            set_flag(ctx.core, owner, "l25_zhiba_boost", boost);
            ctx.start_draw(owner, 1);
            ctx.start_draw(resp, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}
