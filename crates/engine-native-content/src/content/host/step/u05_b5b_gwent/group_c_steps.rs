//! U05-5b Batch C 技能结算步进（`lingji`..`gwjiquan`）。

use super::group_a::refund_turn_limit;
use super::group_c_asks::*;
use super::*;
use crate::event::DamageNature;
use crate::state::SkillInstance;

pub(super) fn step_group_c(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "lingji" => step_lingji(m, ctx),
        "gwjinyan" => step_gwjinyan(m, ctx),
        "gwshenyu" => step_gwshenyu(m, ctx),
        "junchi_sha" => step_junchi_sha(m, ctx),
        "hupeng" => step_hupeng(m, ctx),
        "hupeng3" => step_hupeng3(m.owner, ctx),
        "hupeng4" => step_hupeng4(m, ctx),
        "hunmo" => step_hunmo(m, ctx),
        "shuijian" => step_shuijian(m, ctx),
        "yunhuo" => step_yunhuo(m, ctx),
        "yinzhang" => step_yinzhang(m, ctx),
        "gwtianbian" => step_gwtianbian(m, ctx),
        "gwxiaoshou" => step_gwxiaoshou(m, ctx),
        "kuanglie" => step_kuanglie(m, ctx),
        "gwjiquan" => step_gwjiquan(m, ctx),
        _ => Flow::Done,
    }
}

fn step_lingji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 2);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive
                || !hand_and_equips(ctx.core, owner).into_iter().any(|c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, owner, c)
                })
            {
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_LINGJI_DISCARD,
                build_ask_group_c,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c)
                        if !crate::kernel::is_gw_dudayuanshuai2_locked(
                            ctx.core,
                            ctx.catalog,
                            owner,
                            c,
                        ) =>
                    {
                        Some(c)
                    }
                    _ => None,
                })
                .take(2)
                .collect();
            for &c in &picked {
                discard_he_card(ctx, owner, c);
            }
            if picked.len() == 2 {
                let c0 = &ctx.catalog.cards[picked[0] as usize];
                let c1 = &ctx.catalog.cards[picked[1] as usize];
                if c0.suit == c1.suit {
                    let _ = give_random_spell_of_tier(ctx, owner, 0);
                }
                if c0.rank == c1.rank {
                    let _ = give_random_spell_of_tier(ctx, owner, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_gwjinyan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.card == Some(1) {
        let fid = ctx.current_event_frame_id();
        if let Some(frame) = ctx.res.frame_mut(fid) {
            if let EventData::Damage(d) = &mut frame.data {
                d.prevented = true;
            }
        }
    } else if m.card == Some(2) {
        let _ = give_random_spell_of_tier(ctx, m.owner, 2);
    }
    Flow::Done
}

fn step_gwshenyu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            for c in ctx.core.deck.clone() {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(0, format!("gw_shenyu_vanishtag:{c}")));
            }
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_GWSHENYU_TARGET,
                build_ask_group_c,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            m.target = Some(t);
            let spells = shenyu_discard_spells(ctx.core, ctx.catalog);
            if spells.is_empty() {
                ctx.recover(t, 1, None);
                return Flow::Done;
            }
            if ctx.core.player(t).hp >= ctx.core.player(t).max_hp as i32 {
                return wait_ask(
                    m,
                    ctx,
                    HostStage::SelectCard,
                    ASK_GWSHENYU_PICK,
                    build_ask_group_c,
                );
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectNature,
                ASK_GWSHENYU_MODE,
                build_ask_group_c,
            )
        }
        HostStage::SelectNature => {
            clear_ask_id(ctx.core, owner);
            let Some(t) = m.target else { return Flow::Done };
            if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Tag(0))) {
                ctx.recover(t, 1, None);
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::SelectCard,
                ASK_GWSHENYU_PICK,
                build_ask_group_c,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let (Some(t), Some(ChoiceOption::Card(c))) =
                (m.target, ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
                ctx.core.discard.remove(pos);
                set_flag(ctx.core, 0, &format!("gw_shenyu_vanishtag:{c}"), 1);
                if ctx.core.player(t).alive {
                    ctx.core.player_mut(t).hand.push(c);
                    ctx.emit(EventData::CardsDrawn {
                        player: t,
                        cards: vec![c],
                    });
                } else {
                    ctx.to_discard(c);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_junchi_sha(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(t) = m.target else { return Flow::Done };
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_JUNCHI_USE,
            build_ask_group_c,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c) || !ctx.core.player(t).alive {
                return Flow::Done;
            }
            let stamp = ctx
                .core
                .turn
                .round
                .saturating_mul(16)
                .saturating_add(u32::from(ctx.core.turn.current))
                .saturating_add(1);
            set_flag(ctx.core, owner, "gw_junchi_used_turn", stamp);
            ctx.take_hand_card(owner, c);
            ctx.note_cards_lost(owner, &[c]);
            off_court_put(ctx.core, owner, c);
            ctx.log(LogEvent::SkillTriggered {
                skill: "junchi",
                owner,
                presentation: None,
            });
            m.card = Some(c);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(c) = m.card.take() else { return Flow::Done };
            if !off_court_take(ctx.core, owner, c) {
                return Flow::Done;
            }
            if ctx.core.player(owner).alive && ctx.core.player(t).alive {
                mark_sha_no_count_if_sha(ctx.core, ctx.catalog, owner, c);
                emit_huoge_use(ctx, owner, c, vec![t]);
            } else {
                ctx.to_discard(c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hupeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_HUPENG_CARD,
            build_ask_group_c,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_HUPENG_TARGET,
                build_ask_group_c,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(
                m,
                ctx,
                HostStage::SelectNature,
                ASK_HUPENG_PICK,
                build_ask_group_c,
            )
        }
        HostStage::SelectNature => {
            clear_ask_id(ctx.core, owner);
            let (Some(c), Some(t)) = (m.card, m.target) else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            discard_he_card(ctx, owner, c);
            let name = if tag == 0 {
                "gw_dudayuanshuai1"
            } else {
                "gw_dudayuanshuai2"
            };
            let _ = give_or_draw_card_named(ctx, t, name);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hupeng3(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    let hs2: Vec<CardId> = ctx
        .core
        .player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| ctx.catalog.cards[c as usize].name == "gw_dudayuanshuai2")
        .collect();
    if hs2.is_empty() {
        return Flow::Done;
    }
    let mut others: Vec<CardId> = ctx
        .core
        .player(owner)
        .hand
        .iter()
        .copied()
        .filter(|c| !hs2.contains(c))
        .collect();
    let extra_n = hs2.len();
    for c in hs2 {
        ctx.discard_from_hand(owner, c);
    }
    for _ in 0..extra_n {
        if others.is_empty() {
            break;
        }
        let i = ctx.core.rng.gen_range(others.len() as u32) as usize;
        let c = others.swap_remove(i);
        ctx.discard_from_hand(owner, c);
    }
    Flow::Done
}

fn step_hupeng4(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_HUPENG4_CONFIRM,
            build_ask_group_c,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Card(d1)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(owner).hand.contains(&d1) {
        return Flow::Done;
    }
    vanish_card_from_hand(ctx, owner, d1);
    let fid = ctx.current_event_frame_id();
    let mut gain_card = None;
    if let Some(frame) = ctx.res.frame_mut(fid) {
        if let EventData::CardUsed(u) = &mut frame.data {
            u.targets.retain(|&t| t != owner);
            if u.targets.is_empty() {
                u.cancelled = true;
            }
            gain_card = u.card;
        }
    }
    if ctx.core.player(owner).alive {
        if let Some(gc) = gain_card {
            let taken = if let Some(pos) = ctx.core.table.iter().position(|&x| x == gc) {
                ctx.core.table.remove(pos);
                true
            } else if let Some(pos) = ctx.core.discard.iter().position(|&x| x == gc) {
                ctx.core.discard.remove(pos);
                true
            } else {
                false
            };
            if taken {
                ctx.core.player_mut(owner).hand.push(gc);
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: vec![gc],
                });
            }
        }
    }
    Flow::Done
}

fn step_hunmo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_HUNMO_TARGET,
            build_ask_group_c,
        ),
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            m.target = Some(t);
            if let Some(def) = ctx.catalog.skill_by_name("hunmo2") {
                ctx.insert_skill(
                    (t, def),
                    SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
            if ctx.core.player(owner).hand.len() < 2 {
                let top2 = take_top_deck_cards(ctx.core, 2);
                if top2.is_empty() {
                    return Flow::Done;
                }
                for &c in &top2 {
                    off_court_put(ctx.core, owner, c);
                }
                m.cards = top2;
                return wait_ask(
                    m,
                    ctx,
                    HostStage::SelectCard,
                    ASK_HUNMO_PICK,
                    build_ask_group_c,
                );
            }
            wait_ask(
                m,
                ctx,
                HostStage::CardsAsk,
                ASK_HUNMO_DISCARD_SELF,
                build_ask_group_c,
            )
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let picked = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if m.cards.contains(&c) => c,
                _ => match m.cards.first().copied() {
                    Some(c) => c,
                    None => return Flow::Done,
                },
            };
            m.cards.retain(|&c| c != picked);
            if off_court_take(ctx.core, owner, picked) {
                if ctx.core.player(owner).alive {
                    ctx.core.player_mut(owner).hand.push(picked);
                    ctx.emit(EventData::CardsDrawn {
                        player: owner,
                        cards: vec![picked],
                    });
                } else {
                    ctx.to_discard(picked);
                }
            }
            m.stage = HostStage::SelectNature;
            Flow::Yield
        }
        HostStage::SelectNature => {
            let t = m.target.unwrap_or(owner);
            for c in std::mem::take(&mut m.cards) {
                if off_court_take(ctx.core, owner, c) {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).hand.push(c);
                        ctx.emit(EventData::CardsDrawn {
                            player: t,
                            cards: vec![c],
                        });
                    } else {
                        ctx.to_discard(c);
                    }
                }
            }
            Flow::Done
        }
        HostStage::CardsAsk => {
            clear_ask_id(ctx.core, owner);
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                if ctx.core.player(owner).hand.contains(&c) {
                    ctx.discard_from_hand(owner, c);
                }
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(t) = m.target else { return Flow::Done };
            if !ctx.core.player(t).alive
                || !ctx.core.player(t).hand.iter().any(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, t, c)
                })
            {
                return Flow::Done;
            }
            wait_ask(
                m,
                ctx,
                HostStage::CompareTargetPick,
                ASK_HUNMO_DISCARD_OTHER,
                build_ask_group_c,
            )
        }
        HostStage::CompareTargetPick => {
            clear_ask_id(ctx.core, owner);
            if let (Some(t), Some(ChoiceOption::Card(c))) =
                (m.target, ctx.answer().first().copied())
            {
                if ctx.core.player(t).hand.contains(&c) {
                    ctx.discard_from_hand(t, c);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_shuijian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_SHUIJIAN_COST,
            build_ask_group_c,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    ctx.discard_from_hand(owner, c);
    let targets = enemies_of(ctx.core, ctx.catalog, owner);
    if !targets.is_empty() {
        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
            card: None,
            player: owner,
            targets,
            nature: DamageNature::Normal,
            use_as: Some(CardKind::ArcheryAttack),
            cancelled: false,
        }));
    }
    Flow::Done
}

fn step_yunhuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        let round = ctx.core.turn.round;
        set_flag(ctx.core, owner, "gw_yunhuo_fired_round", round);
        set_flag(ctx.core, owner, "extra_turn_insert_queued", 1);
        m.targets = enemies_of(ctx.core, ctx.catalog, owner);
        m.stage = HostStage::CompareTargetPick;
    }
    while !m.targets.is_empty() {
        let e = m.targets.remove(0);
        if !ctx.core.player(e).alive {
            continue;
        }
        if !ctx.core.player(e).hand.is_empty() {
            let drop_pool: Vec<CardId> = ctx
                .core
                .player(e)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, e, c)
                })
                .collect();
            if !drop_pool.is_empty() {
                let idx = ctx.core.rng.gen_range(drop_pool.len() as u32) as usize;
                ctx.discard_from_hand(e, drop_pool[idx]);
                return Flow::Yield;
            }
        } else {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                e,
                1,
                DamageNature::Fire,
                None,
            )));
            return Flow::Yield;
        }
    }
    Flow::Done
}

fn step_yinzhang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_YINZHANG_COST,
            build_ask_group_c,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            discard_he_card(ctx, owner, c);
            let mut pool: Vec<usize> = (0..SILVER_SPELL_PICK_NAMES.len()).collect();
            let mut n = 0u32;
            while n < 3 && !pool.is_empty() {
                let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                let tag = pool.swap_remove(i);
                set_flag(ctx.core, owner, &format!("gw_yinzhang_opt:{n}"), tag as u32 + 1);
                n += 1;
            }
            set_flag(ctx.core, owner, "gw_yinzhang_n", n);
            wait_ask(
                m,
                ctx,
                HostStage::SelectNature,
                ASK_YINZHANG_PICK,
                build_ask_group_c,
            )
        }
        HostStage::SelectNature => {
            clear_ask_id(ctx.core, owner);
            if let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() {
                if let Some(&name) = SILVER_SPELL_PICK_NAMES.get(tag as usize) {
                    let _ = give_or_draw_card_named(ctx, owner, name);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_gwtianbian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::DrawSettle {
        if let Some(fid) = m.card.take() {
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, format!("taoyuan_include_owner:{fid}")));
        }
        return Flow::Done;
    }
    if m.stage == HostStage::Act {
        return wait_ask(
            m,
            ctx,
            HostStage::SelectNature,
            ASK_GWTIANBIAN_MODE,
            build_ask_group_c,
        );
    }
    clear_ask_id(ctx.core, owner);
    let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if tag == 0 {
        let kind = if ctx.core.rng.gen_range(2) == 0 {
            CardKind::Taoyuan
        } else {
            CardKind::Wugu
        };
        let targets: Vec<PlayerId> = ctx
            .core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                ctx.core.player(p).alive && (p == owner || !is_target_hidden(ctx.core, p))
            })
            .collect();
        if !targets.is_empty() {
            if kind == CardKind::Taoyuan && targets.contains(&owner) {
                let frame_id = ctx.res.next_event_id;
                ctx.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("taoyuan_include_owner:{frame_id}")), 1);
                m.card = Some(frame_id);
                m.stage = HostStage::DrawSettle;
            }
            ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                card: None,
                player: owner,
                targets,
                nature: DamageNature::Normal,
                use_as: Some(kind),
                cancelled: false,
            }));
            if kind == CardKind::Taoyuan && m.stage == HostStage::DrawSettle {
                return Flow::Yield;
            }
        }
    } else {
        let kind = if ctx.core.rng.gen_range(2) == 0 {
            CardKind::SavageAttack
        } else {
            CardKind::ArcheryAttack
        };
        let targets: Vec<PlayerId> = ctx
            .core
            .seat_order_from(ctx.core.next_alive(owner))
            .into_iter()
            .filter(|&p| {
                p != owner && ctx.core.player(p).alive && !is_target_hidden(ctx.core, p)
            })
            .collect();
        if !targets.is_empty() {
            ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                card: None,
                player: owner,
                targets,
                nature: DamageNature::Normal,
                use_as: Some(kind),
                cancelled: false,
            }));
        }
    }
    Flow::Done
}

fn step_gwxiaoshou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_GWXIAOSHOU_COST,
            build_ask_group_c,
        ),
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_GWXIAOSHOU_TARGET,
                build_ask_group_c,
            )
        }
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let (Some(c), Some(ChoiceOption::Player(t))) = (m.card, ctx.answer().first().copied())
            else {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            };
            let used = flag(ctx.core, owner, "gw_use2_cnt:gwxiaoshou");
            set_flag(ctx.core, owner, "gw_use2_cnt:gwxiaoshou", used + 1);
            discard_he_card(ctx, owner, c);
            if ctx.core.player(t).alive {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_kuanglie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::DrawSettle {
        m.stage = HostStage::CompareTargetPick;
        if ctx.core.player(owner).alive {
            ctx.start_draw(owner, 1);
            return Flow::Yield;
        }
    }
    while !m.targets.is_empty() {
        let t = m.targets.remove(0);
        if !ctx.core.player(t).alive {
            continue;
        }
        let he: Vec<CardId> = hand_and_equips(ctx.core, t)
            .into_iter()
            .filter(|&c| {
                !crate::kernel::is_gw_dudayuanshuai2_locked(ctx.core, ctx.catalog, t, c)
            })
            .collect();
        if he.is_empty() {
            continue;
        }
        let c = he[ctx.core.rng.gen_range(he.len() as u32) as usize];
        discard_he_card(ctx, t, c);
        let cnt = flag(ctx.core, owner, "gw_kuanglie_count") + 1;
        set_flag(ctx.core, owner, "gw_kuanglie_count", cnt);
        m.stage = if cnt % 2 == 0 {
            HostStage::DrawSettle
        } else {
            HostStage::CompareTargetPick
        };
        return Flow::Yield;
    }
    Flow::Done
}

fn step_gwjiquan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::TargetChoice,
            ASK_GWJIQUAN_TARGETS,
            build_ask_group_c,
        ),
        HostStage::TargetChoice => {
            clear_ask_id(ctx.core, owner);
            let picked: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                refund_turn_limit(ctx, owner, &m.def_key);
                return Flow::Done;
            }
            m.targets = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|p| picked.contains(p))
                .collect();
            advance_gwjiquan_queue(m, ctx)
        }
        HostStage::SelectCard => {
            clear_ask_id(ctx.core, owner);
            let (Some(t), Some(ChoiceOption::Card(c))) =
                (m.target.take(), ctx.answer().first().copied())
            else {
                return advance_gwjiquan_queue(m, ctx);
            };
            if ctx.core.player(t).hand.contains(&c) {
                ctx.give_hand_cards(t, owner, &[c]);
            } else if ctx.core.player(t).equips.contains(&c) {
                ctx.core.player_mut(t).equips.retain(|&x| x != c);
                ctx.core.declared_slots.remove(&c);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                ctx.note_cards_lost(t, &[c]);
                ctx.core.player_mut(owner).hand.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: t,
                    to: owner,
                });
            }
            if ctx.core.player(t).alive && ctx.core.player(owner).alive {
                set_flag(ctx.core, t, "vm_va_sha_no_count", 1);
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: t,
                    targets: vec![owner],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
                return Flow::Yield;
            }
            advance_gwjiquan_queue(m, ctx)
        }
        HostStage::DrawSettle => advance_gwjiquan_queue(m, ctx),
        _ => Flow::Done,
    }
}

fn advance_gwjiquan_queue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while !m.targets.is_empty() {
        let t = m.targets.remove(0);
        if !ctx.core.player(t).alive || hand_and_equips(ctx.core, t).is_empty() {
            continue;
        }
        m.target = Some(t);
        return wait_ask(
            m,
            ctx,
            HostStage::SelectCard,
            ASK_GWJIQUAN_STEAL,
            build_ask_group_c,
        );
    }
    Flow::Done
}
