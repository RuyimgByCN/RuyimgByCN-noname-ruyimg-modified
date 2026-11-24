//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 濒死/死亡/弃失去牌/傲才响应执行（15 技：`dying_lose_resp.rs`）。

use super::asks_b::*;
use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Faction};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_dying_lose_resp(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "aocai" => step_aocai(m, ctx),
        "buqu" => step_buqu(m, ctx),
        "chenqing" => step_chenqing(m, ctx),
        "dualside" => step_dualside(m, ctx),
        "fenji" => step_fenji(m, ctx),
        "liangzhu" => step_liangzhu(m, ctx),
        "lirang" => step_lirang(m, ctx),
        "olniepan" => step_olniepan(m, ctx),
        "relianying" => step_relianying(m, ctx),
        "renshe" => step_renshe(m, ctx),
        "requanji" => step_requanji(m, ctx),
        "retuntian" => step_retuntian(m, ctx),
        "rexingshang" => step_rexingshang(m, ctx),
        "rexuanfeng" => step_rexuanfeng(m, ctx),
        "reqizhou" => {
            sync_reqizhou_skills(ctx.core, ctx.catalog, m.owner);
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

pub(in crate::content::host) fn aocai_respond_skill_options(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    need: CardKind,
    _is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    if core.turn.current == player || !core.player(player).alive {
        return;
    }
    if !matches!(
        need,
        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
    ) {
        return;
    }
    if core.deck.is_empty() && core.discard.is_empty() {
        return;
    }
    let Some(def) = catalog.skill_by_name("aocai") else {
        return;
    };
    if core.has_skill(player, def)
        && is_unblocked_npc_a(core, catalog, player)
        && !out.contains(&ChoiceOption::Skill(def))
    {
        out.push(ChoiceOption::Skill(def));
    }
}

pub(in crate::content::host) fn aocai_try_activate_respond_skill(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: crate::ids::SkillDefId,
    need: CardKind,
    fid: crate::ids::EventId,
    is_use: bool,
) -> Option<bool> {
    if ctx.catalog.skill_name(def) != "aocai"
        || ctx.core.turn.current == player
        || !ctx.core.player(player).alive
        || !matches!(
            need,
            CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
        )
    {
        return None;
    }
    if flag(ctx.core, player, &format!("l11_aocai_tried:{fid}")) > 0 {
        return None;
    }
    set_flag(ctx.core, player, &format!("l11_aocai_tried:{fid}"), 1);
    let dying_self = need == CardKind::Tao
        && ctx
            .res
            .stack
            .iter()
            .rev()
            .any(|f| matches!(f.data, EventData::Dying { player: dp, .. } if dp == player));
    let need_tag = if need == CardKind::Shan {
        1
    } else if need.is_sha_like() {
        2
    } else if need == CardKind::Tao && dying_self {
        5
    } else {
        4
    };
    set_flag(ctx.core, player, "u01_respond_fid", fid);
    set_flag(ctx.core, player, "l11_aocai_is_use", u32::from(is_use));
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger { frame: fid },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "aocai".to_string(),
            behavior: HostBehavior::L11CrossNpcA,
            owner: player,
            target: None,
            card: None,
            cards: vec![need_tag],
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
    Some(false)
}

fn aocai_matches_need(catalog: &crate::catalog::Catalog, need_tag: u32, cid: u32) -> bool {
    let k = catalog.card_kind(cid);
    match need_tag {
        1 => k == CardKind::Shan,
        2 => k.is_sha_like(),
        4 => k == CardKind::Tao,
        5 => matches!(k, CardKind::Tao | CardKind::Jiu),
        _ => false,
    }
}

fn step_aocai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let need_tag = m.cards.first().copied().unwrap_or(0);
            let cnt = if ctx.core.player(owner).hand.is_empty() {
                4
            } else {
                2
            };
            let mut peek = Vec::with_capacity(cnt);
            for _ in 0..cnt {
                if let Some(c) = pop_deck_reshuffle(ctx) {
                    peek.push(c);
                }
            }
            if !peek
                .iter()
                .any(|&c| aocai_matches_need(ctx.catalog, need_tag, c))
            {
                for &c in peek.iter().rev() {
                    put_on_top_of_deck(ctx, c);
                }
                return finish_respond_park(ctx, owner, false);
            }
            ctx.core
                .vm_aux
                .piles
                .insert((owner, "l11_aocai_peek".to_string()), peek);
            m.cards = vec![need_tag];
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_AOCAI_CHOOSE_CARD)
        }
        HostStage::SelectCard => {
            let need_tag = m.cards.first().copied().unwrap_or(0);
            let is_use = flag(ctx.core, owner, "l11_aocai_is_use") > 0;
            let peek = ctx
                .core
                .vm_aux
                .piles
                .remove(&(owner, "l11_aocai_peek".to_string()))
                .unwrap_or_default();
            let picked = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|&c| {
                    peek.contains(&c) && aocai_matches_need(ctx.catalog, need_tag, c)
                });
            for &c in peek.iter().rev() {
                if Some(c) != picked {
                    put_on_top_of_deck(ctx, c);
                }
            }
            if let Some(c) = picked {
                if is_use {
                    ctx.core.table.push(c);
                } else {
                    ctx.to_discard(c);
                }
                set_flag(ctx.core, owner, "u01_respond_card_override", c + 1);
                if need_tag == 1 && is_use {
                    set_flag(ctx.core, owner, "u01_respond_emit_use", 1);
                }
                finish_respond_park(ctx, owner, true)
            } else {
                finish_respond_park(ctx, owner, false)
            }
        }
        _ => finish_respond_park(ctx, owner, false),
    }
}

fn step_buqu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let fid = ctx.res.stack.last().map_or(0, |f| f.id);
            set_flag(ctx.core, owner, &format!("l11_buqu_dying:{fid}"), 1);
            let Some(cid) = pop_deck_reshuffle(ctx) else {
                return clear_and_done(m);
            };
            let rank = effective_card_rank(ctx.catalog, cid);
            let pile = ctx
                .core
                .vm_aux
                .piles
                .entry((owner, "l11_buqu_chuang".to_string()))
                .or_default();
            let dup = pile
                .iter()
                .any(|&x| effective_card_rank(ctx.catalog, x) == rank);
            if dup {
                ctx.out.push(LogEvent::CardsRevealed {
                    cards: vec![cid],
                    to: ctx.core.alive_players().collect(),
                });
                ctx.to_discard(cid);
                return clear_and_done(m);
            }
            pile.push(cid);
            let len = pile.len() as u32;
            let mut show = pile.clone();
            ctx.out.push(LogEvent::CardsRevealed {
                cards: std::mem::take(&mut show),
                to: ctx.core.alive_players().collect(),
            });
            set_flag(ctx.core, owner, "l11_mod:buqu_chuang", len);
            let cur_hp = ctx.core.player(owner).hp;
            if cur_hp <= 0 {
                emit_recover(ctx, owner, (1 - cur_hp) as u32);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_chenqing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_CHENQING_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            set_flag(ctx.core, owner, "l11_chenqing_round", ctx.core.turn.round);
            m.targets = vec![t];
            if ctx.core.player(t).alive {
                ctx.start_draw(t, 4);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => {
            if let Some(&t) = m.targets.first() {
                if ctx.core.player(t).alive && !he_cards(ctx.core, t).is_empty() {
                    return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_CHENQING_DISCARD);
                }
            }
            clear_and_done(m)
        }
        HostStage::SelectCard => {
            let Some(&t) = m.targets.first() else {
                return clear_and_done(m);
            };
            let cs = answered_cards(ctx);
            let mut mask = 0u32;
            if cs.len() == 4 {
                for &c in &cs {
                    mask |= suit_bit(card_suit(ctx.catalog, ctx.core, t, c));
                }
            }
            discard_player_cards(ctx, t, &cs);
            if mask == 0b1111 {
                if let Some(dying_p) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: t,
                        targets: vec![dying_p],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Tao),
                        cancelled: false,
                    }));
                    return clear_and_yield(m, HostStage::ShuchuiTarget);
                }
            }
            clear_and_done(m)
        }
        HostStage::ShuchuiTarget => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_dualside(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage == HostStage::Act {
        execute_dualside_switch(ctx, m.owner, false);
    }
    clear_and_done(m)
}

fn step_fenji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_FENJI_CONFIRM),
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) && ctx.core.player(owner).alive {
                emit_lose_hp(ctx, owner, 1);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => {
            if let Some(victim) = m.target.filter(|&p| ctx.core.player(p).alive) {
                ctx.start_draw(victim, 2);
                return clear_and_yield(m, HostStage::ShuchuiTarget);
            }
            clear_and_done(m)
        }
        HostStage::ShuchuiTarget => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_liangzhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_LIANGZHU_MODE),
        HostStage::SelectNature => {
            match answered_tag(ctx) {
                Some(0) if ctx.core.player(owner).alive => {
                    ctx.start_draw(owner, 1);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
                Some(1) => {
                    if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                        ctx.start_draw(t, 2);
                        return clear_and_yield(m, HostStage::DrawSettle);
                    }
                }
                _ => {}
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_lirang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            m.cards.retain(|c| ctx.core.discard.contains(c));
            if m.cards.is_empty() {
                return clear_and_done(m);
            }
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_LIRANG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_LIRANG_CARDS)
        }
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let mut gained = Vec::new();
                for c in cs {
                    if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
                        ctx.core.discard.remove(pos);
                        m.cards.retain(|&x| x != c);
                        gained.push(c);
                    }
                }
                if !gained.is_empty() {
                    let delivered =
                        (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, t, &gained);
                    ctx.core
                        .player_mut(t)
                        .hand
                        .extend(delivered.iter().copied());
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: t,
                            cards: delivered,
                        });
                    }
                }
            }
            m.cards.retain(|c| ctx.core.discard.contains(c));
            if !m.cards.is_empty() && !alive_others(ctx.core, m.owner).is_empty() {
                return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_LIRANG_TARGET);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_olniepan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_OLNIEPAN_CONFIRM),
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            set_flag(ctx.core, owner, "l11_olniepan_used", 1);
            let all_cards = hej_cards(ctx.core, owner);
            if !all_cards.is_empty() {
                discard_player_cards(ctx, owner, &all_cards);
            }
            ctx.core.player_mut(owner).face_down = false;
            ctx.core.player_mut(owner).linked = false;
            ctx.start_draw(owner, 3);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => {
            let cur = ctx.core.player(owner).hp;
            let tgt = 3i32.min(ctx.core.player(owner).max_hp as i32);
            if cur < tgt && ctx.core.player(owner).alive {
                emit_recover(ctx, owner, (tgt - cur) as u32);
                return clear_and_yield(m, HostStage::SelectNature);
            }
            if ["bazhen", "olhuoji", "olkanpo"]
                .iter()
                .any(|s| !has_npc_a_skill(ctx.core, ctx.catalog, owner, s))
            {
                wait_npc_ask(m, ctx, HostStage::HuanrenPick, ASK_OLNIEPAN_SKILL)
            } else {
                clear_and_done(m)
            }
        }
        HostStage::SelectNature => {
            if ["bazhen", "olhuoji", "olkanpo"]
                .iter()
                .any(|s| !has_npc_a_skill(ctx.core, ctx.catalog, owner, s))
            {
                wait_npc_ask(m, ctx, HostStage::HuanrenPick, ASK_OLNIEPAN_SKILL)
            } else {
                clear_and_done(m)
            }
        }
        HostStage::HuanrenPick => {
            let sk = match answered_tag(ctx) {
                Some(0) => Some("bazhen"),
                Some(1) => Some("olhuoji"),
                Some(2) => Some("olkanpo"),
                _ => None,
            };
            if let Some(name) = sk {
                ensure_perm_skill(ctx, owner, name);
                crate::resolution::sync_trigger_owners(ctx.core, ctx.res);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn relianying_draw_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(t).alive {
            ctx.start_draw(t, 1);
            return clear_and_yield(m, HostStage::DrawSettle);
        }
    }
    clear_and_done(m)
}

fn step_relianying(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let fid = ctx.res.stack.last().map_or(0, |f| f.id);
            set_flag(ctx.core, owner, "l11_relianying_last_fid", fid);
            let n = flag(ctx.core, owner, "l11_batch_lost_hand").max(1);
            set_flag(ctx.core, owner, "l11_batch_lost_hand", 0);
            m.cards = vec![n];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_RELIANYING_TARGETS)
        }
        HostStage::TargetChoice => {
            m.targets = answered_players(ctx);
            relianying_draw_next(m, ctx)
        }
        HostStage::DrawSettle => relianying_draw_next(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_renshe(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_RENSHE_MODE),
        HostStage::SelectNature => match answered_tag(ctx) {
            Some(0) => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_RENSHE_DRAW_TARGET),
            Some(1) => {
                inc_flag(ctx.core, owner, "l11_waishi_bonus", 1);
                inc_flag(ctx.core, owner, "l11_renshe_waishi_bonus", 1);
                clear_and_done(m)
            }
            Some(2) => {
                wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_RENSHE_FACTION_TARGET)
            }
            _ => clear_and_done(m),
        },
        HostStage::TargetChoice => {
            if let Some(t) = answered_players(ctx).into_iter().next() {
                m.target = Some(t);
                if ctx.core.player(owner).alive {
                    ctx.start_draw(owner, 1);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target.take().filter(|&p| ctx.core.player(p).alive) {
                ctx.start_draw(t, 1);
                return clear_and_yield(m, HostStage::HuanrenPick);
            }
            clear_and_done(m)
        }
        HostStage::HuanrenPick => clear_and_done(m),
        HostStage::ShuchuiTarget => {
            if let Some(t) = answered_players(ctx).into_iter().next() {
                let code = match ctx.core.effective_faction(ctx.catalog, t) {
                    Some(Faction::Wei) => 1,
                    Some(Faction::Shu) => 2,
                    Some(Faction::Wu) => 3,
                    Some(Faction::Qun) => 4,
                    _ => 0,
                };
                if code > 0 {
                    set_flag(ctx.core, owner, "l11_faction_override", code);
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_requanji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let count = if ctx
                .res
                .stack
                .last()
                .is_some_and(|f| {
                    matches!(f.data, EventData::Damage(_) | EventData::DamageWindow { .. })
                })
            {
                m.cards.first().copied().unwrap_or(1).max(1)
            } else {
                1
            };
            m.cards = vec![count];
            wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REQUANJI_CONFIRM)
        }
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) || !ctx.core.player(owner).alive {
                return clear_and_done(m);
            }
            let rem = m.cards.first().copied().unwrap_or(1).saturating_sub(1);
            m.cards = vec![rem];
            ctx.start_draw(owner, 1);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).hand.is_empty() && ctx.core.player(owner).alive {
                return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REQUANJI_CARD);
            }
            if m.cards.first().copied().unwrap_or(0) > 0 && ctx.core.player(owner).alive {
                return wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REQUANJI_CONFIRM);
            }
            clear_and_done(m)
        }
        HostStage::SelectCard => {
            if let Some(cid) = answered_cards(ctx).into_iter().next() {
                if ctx.core.player(owner).hand.contains(&cid) {
                    ctx.take_hand_card(owner, cid);
                    ctx.note_cards_lost(owner, &[cid]);
                    let pile = ctx
                        .core
                        .vm_aux
                        .piles
                        .entry((owner, "l11_requanji_quan".to_string()))
                        .or_default();
                    pile.push(cid);
                    let len = pile.len() as u32;
                    set_flag(ctx.core, owner, "l11_mod:requanji_quan", len);
                }
            }
            if m.cards.first().copied().unwrap_or(0) > 0 && ctx.core.player(owner).alive {
                return wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REQUANJI_CONFIRM);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_retuntian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let fid = ctx.res.stack.last().map_or(0, |f| f.id);
            set_flag(ctx.core, owner, "l11_retuntian_last_fid", fid);
            wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_RETUNTIAN_CONFIRM)
        }
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            clear_and_yield(m, HostStage::SelectNature);
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::SelectNature => {
            let suit = flag(ctx.core, owner, "last_judge_suit");
            let jcard = flag(ctx.core, owner, "last_judge_card");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            if ctx.core.discard.contains(&jcard) && take_from_table_discard_or_equip(ctx, jcard) {
                if suit == 2 {
                    let delivered =
                        (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &[jcard]);
                    ctx.core
                        .player_mut(owner)
                        .hand
                        .extend(delivered.iter().copied());
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: owner,
                            cards: delivered,
                        });
                    }
                } else {
                    let pile = ctx
                        .core
                        .vm_aux
                        .piles
                        .entry((owner, "l11_retuntian_tian".to_string()))
                        .or_default();
                    pile.push(jcard);
                    let len = pile.len() as u32;
                    set_flag(ctx.core, owner, "l11_mod:retuntian_tian", len);
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_rexingshang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_REXINGSHANG_MODE),
        HostStage::SelectNature => match answered_tag(ctx) {
            Some(0) => {
                if let Some(dead) = m.target {
                    let cs = he_cards(ctx.core, dead);
                    if !cs.is_empty() {
                        move_cards_to_hand(ctx, dead, owner, &cs);
                    }
                }
                clear_and_done(m)
            }
            Some(1) => {
                emit_recover(ctx, owner, 1);
                clear_and_yield(m, HostStage::DrawSettle)
            }
            _ => clear_and_done(m),
        },
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_rexuanfeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_REXUANFENG_MODE),
        HostStage::SelectNature => match answered_tag(ctx) {
            Some(0) => {
                m.cards = vec![2];
                wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REXUANFENG_DISC_TARGET)
            }
            Some(1) => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REXUANFENG_MOVE_FROM),
            _ => clear_and_done(m),
        },
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REXUANFENG_DISC_CARD)
        }
        HostStage::SelectCard => {
            if let (Some(t), Some(c)) = (m.target, answered_cards(ctx).into_iter().next()) {
                discard_player_cards(ctx, t, &[c]);
            }
            let rem = m.cards.first().copied().unwrap_or(0).saturating_sub(1);
            m.cards = vec![rem];
            if rem > 0
                && alive_others(ctx.core, owner)
                    .into_iter()
                    .any(|p| !he_cards(ctx.core, p).is_empty())
            {
                return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REXUANFENG_DISC_TARGET);
            }
            clear_and_done(m)
        }
        HostStage::JiefenTarget => {
            let Some(from) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(from);
            wait_npc_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_REXUANFENG_MOVE_CARD)
        }
        HostStage::JiefenTargetDiscard => {
            let Some(eq) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![eq];
            wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_REXUANFENG_MOVE_TO)
        }
        HostStage::ShuchuiTarget => {
            if let (Some(from), Some(to), Some(&eq)) = (
                m.target,
                answered_players(ctx).into_iter().next(),
                m.cards.first(),
            ) {
                move_card_ej(ctx, from, to, eq);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}
