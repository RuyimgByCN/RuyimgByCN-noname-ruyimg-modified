//! Step handlers for Batch C part 1:
//! `ddddongcha`, `dddzhijie`, `dddqiahua`, `dddxunxun`.

use super::*;

pub(super) fn step_c1(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    match machine.def_key.as_str() {
        "ddddongcha" => Some(step_ddddongcha(machine, ctx)),
        "dddzhijie" => Some(step_dddzhijie(machine, ctx)),
        "dddqiahua" => Some(step_dddqiahua(machine, ctx)),
        "dddxunxun" => Some(step_dddxunxun(machine, ctx)),
        _ => None,
    }
}

fn step_ddddongcha(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    let is_turn_end = machine.card == Some(1);
    let Some(target) = machine.target else {
        return Flow::Done;
    };
    if target == owner || !ctx.core.player(target).alive {
        return Flow::Done;
    }
    if !is_turn_end {
        match machine.stage {
            HostStage::Act => {
                if ctx.core.player(owner).hand.is_empty()
                    || ctx.core.player(target).hand.is_empty()
                {
                    return Flow::Done;
                }
                wait_ask_2(machine, ctx, HostStage::TargetChoice, 2029)
            }
            HostStage::TargetChoice => {
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    || ctx.core.player(owner).hand.is_empty()
                    || ctx.core.player(target).hand.is_empty()
                {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "ddddongcha",
                    owner,
                    presentation: None,
                });
                set_flag(
                    ctx.core,
                    owner,
                    "ddddongcha_active_turn",
                    u32::from(target) + 1,
                );
                wait_ask_2(machine, ctx, HostStage::SelectCard, 2030)
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if ctx.core.player(owner).hand.contains(&c) {
                        ctx.take_hand_card(owner, c);
                        ctx.note_cards_lost(owner, &[c]);
                        ctx.core
                            .vm_aux
                            .piles
                            .entry((owner, "ddd_jian".to_string()))
                            .or_default()
                            .push(c);
                        ctx.emit(EventData::CardsLost {
                            player: owner,
                            cards: vec![c],
                            origins: vec![LostOrigin::Hand],
                        });
                    }
                }
                if !ctx.core.player(target).hand.is_empty() {
                    wait_ask_2(machine, ctx, HostStage::CardsAsk, 2031)
                } else {
                    Flow::Done
                }
            }
            HostStage::CardsAsk => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if ctx.core.player(target).hand.contains(&c) {
                        ctx.take_hand_card(target, c);
                        ctx.note_cards_lost(target, &[c]);
                        ctx.core
                            .vm_aux
                            .piles
                            .entry((owner, "ddd_jian".to_string()))
                            .or_default()
                            .push(c);
                        ctx.emit(EventData::CardsLost {
                            player: target,
                            cards: vec![c],
                            origins: vec![LostOrigin::Hand],
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match machine.stage {
            HostStage::Act => {
                if flag(ctx.core, owner, "ddddongcha_active_turn") != u32::from(target) + 1
                    || ddd_jian_cards(ctx.core, owner).is_empty()
                {
                    return Flow::Done;
                }
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "ddddongcha_active_turn".to_string()));
                ctx.log(LogEvent::SkillTriggered {
                    skill: "ddddongcha",
                    owner,
                    presentation: None,
                });
                wait_ask_2(machine, ctx, HostStage::SelectCard, 2032)
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if let Some(pile) = ctx
                        .core
                        .vm_aux
                        .piles
                        .get_mut(&(owner, "ddd_jian".to_string()))
                    {
                        if let Some(pos) = pile.iter().position(|&x| x == c) {
                            pile.remove(pos);
                            ctx.core.player_mut(owner).hand.push(c);
                            ctx.log(LogEvent::CardsGivenPublic {
                                from: owner,
                                to: owner,
                                cards: vec![c],
                            });
                        }
                    }
                }
                if !ddd_jian_cards(ctx.core, owner).is_empty() && ctx.core.player(target).alive {
                    wait_ask_2(machine, ctx, HostStage::CardsAsk, 2033)
                } else {
                    Flow::Done
                }
            }
            HostStage::CardsAsk => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if let Some(pile) = ctx
                        .core
                        .vm_aux
                        .piles
                        .get_mut(&(owner, "ddd_jian".to_string()))
                    {
                        if let Some(pos) = pile.iter().position(|&x| x == c) {
                            pile.remove(pos);
                            ctx.core.player_mut(target).hand.push(c);
                            ctx.log(LogEvent::CardsGivenPublic {
                                from: owner,
                                to: target,
                                cards: vec![c],
                            });
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

fn step_dddzhijie(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return finish_respond_park_pub(ctx, owner, false);
    }
    let need_code = machine.card.unwrap_or(0);
    let is_nullify = need_code == 3;
    let is_use = machine.targets.first().copied() == Some(1);
    match machine.stage {
        HostStage::Act => {
            set_flag(
                ctx.core,
                owner,
                "dddzhijie_need_diff_color",
                u32::from(is_nullify),
            );
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2034)
        }
        HostStage::SelectCard => {
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "dddzhijie_need_diff_color".to_string()));
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picks.len() != 2 {
                return finish_respond_park_pub(ctx, owner, false);
            }
            let same_color = effective_is_red(ctx.core, ctx.catalog, owner, picks[0])
                == effective_is_red(ctx.core, ctx.catalog, owner, picks[1]);
            let valid = if is_nullify { !same_color } else { same_color };
            if !valid {
                return finish_respond_park_pub(ctx, owner, false);
            }
            let removed_ok = if let Some(pile) = ctx
                .core
                .vm_aux
                .piles
                .get_mut(&(owner, "ddd_jian".to_string()))
            {
                if !picks.iter().all(|c| pile.contains(c)) {
                    false
                } else {
                    for &c in &picks {
                        pile.retain(|&x| x != c);
                    }
                    true
                }
            } else {
                false
            };
            if !removed_ok {
                return finish_respond_park_pub(ctx, owner, false);
            }
            for &c in &picks {
                ctx.to_discard(c);
            }
            ctx.log(LogEvent::CardDiscarded {
                player: owner,
                cards: picks,
            });
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddzhijie",
                owner,
                presentation: None,
            });
            if is_use {
                if !is_nullify {
                    set_flag(ctx.core, owner, "u01_respond_emit_use", 1);
                }
                let cur = flag(ctx.core, owner, "dddzhijie_pending_draw");
                set_flag(ctx.core, owner, "dddzhijie_pending_draw", cur + 1);
            }
            finish_respond_park_pub(ctx, owner, true)
        }
        _ => Flow::Done,
    }
}

fn step_dddqiahua(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            let Some(tgt) = machine.target else {
                return Flow::Done;
            };
            let hp = ctx.core.player(tgt).hp;
            if tgt == owner
                || !ctx.core.player(tgt).alive
                || hp <= 0
                || dddfusi_unshown_hand(ctx.core, owner).len() < hp as usize
            {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2036)
        }
        HostStage::SelectCard => {
            let Some(tgt) = machine.target else {
                return Flow::Done;
            };
            let need = ctx.core.player(tgt).hp.max(1) as usize;
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picks.len() != need || !ctx.core.player(tgt).alive {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddqiahua",
                owner,
                presentation: None,
            });
            for &c in &picks {
                set_flag(ctx.core, owner, &format!("u01_shown_card:{c}"), 1);
            }
            ctx.log(LogEvent::CardsRevealed {
                cards: picks,
                to: ctx.core.alive_players().collect(),
            });
            if let Some(xun_def) = ctx.catalog.skill_by_name("dddxunxun") {
                if !ctx.core.has_skill(tgt, xun_def) {
                    ctx.core.skills.insert(
                        (tgt, xun_def),
                        crate::state::SkillInstance {
                            owner: tgt,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                    set_flag(ctx.core, tgt, "dddqiahua_temp_xunxun", 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddxunxun(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2037),
        HostStage::TargetChoice => {
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            if ctx.core.deck.len() < 4 && !ctx.core.discard.is_empty() {
                let mut reshuffled = std::mem::take(&mut ctx.core.discard);
                ctx.core.rng.shuffle(&mut reshuffled);
                crate::kernel::elo_on_wash_deck(ctx.core, &reshuffled);
                reshuffled.extend(std::mem::take(&mut ctx.core.deck));
                ctx.core.deck = reshuffled;
                ctx.log(LogEvent::DeckReshuffled);
            }
            let n = ctx.core.deck.len().min(4);
            if n == 0 {
                return Flow::Done;
            }
            let len = ctx.core.deck.len();
            let top: Vec<CardId> = ctx.core.deck[len - n..].iter().copied().rev().collect();
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddxunxun",
                owner,
                presentation: None,
            });
            machine.cards = top;
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2038)
        }
        HostStage::SelectCard => {
            let mut top_picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            let orig_all = machine.cards.clone();
            let remaining: Vec<CardId> = orig_all
                .iter()
                .copied()
                .filter(|c| !top_picks.contains(c))
                .collect();
            machine.cards = remaining.clone();
            // Stash chosen top picks into machine.targets? No, cards > u8! Store in vm_aux flags cleanly!
            set_flag(ctx.core, owner, "dddxunxun_top_len", top_picks.len() as u32);
            for (i, &c) in top_picks.iter().enumerate() {
                set_flag(ctx.core, owner, &format!("dddxunxun_top:{i}"), c);
            }
            if remaining.is_empty() {
                apply_dddxunxun_deck_order(ctx, owner, &orig_all, &top_picks, &[]);
                top_picks.clear();
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::CardsAsk, 2039)
        }
        HostStage::CardsAsk => {
            let bot_picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            let n = flag(ctx.core, owner, "dddxunxun_top_len") as usize;
            let mut top_picks = Vec::with_capacity(n);
            for i in 0..n {
                let c = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("dddxunxun_top:{i}")))
                    .unwrap_or(0);
                top_picks.push(c);
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "dddxunxun_top_len".to_string()));
            let mut all_peeked = top_picks.clone();
            all_peeked.extend(machine.cards.iter().copied());
            apply_dddxunxun_deck_order(ctx, owner, &all_peeked, &top_picks, &bot_picks);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn apply_dddxunxun_deck_order(
    ctx: &mut Ctx<'_>,
    _owner: PlayerId,
    all_peeked: &[CardId],
    top_picks: &[CardId],
    bot_picks: &[CardId],
) {
    ctx.core.deck.retain(|c| !all_peeked.contains(c));
    if !bot_picks.is_empty() {
        let mut prefix: Vec<CardId> = bot_picks.iter().copied().rev().collect();
        prefix.extend(std::mem::take(&mut ctx.core.deck));
        ctx.core.deck = prefix;
    }
    ctx.core.deck.extend(top_picks.iter().copied().rev());
}
