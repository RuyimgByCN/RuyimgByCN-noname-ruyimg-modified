//! VM 排队效果在 Halted 后的权威状态落账。

use super::*;

fn pop_deck(ctx: &mut Ctx<'_>) -> Option<CardId> {
    if ctx.core.deck.is_empty() {
        if ctx.core.discard.is_empty() {
            return None;
        }
        let mut cards = std::mem::take(&mut ctx.core.discard);
        ctx.core.rng.shuffle(&mut cards);
        ctx.core.deck = cards;
        ctx.out.push(LogEvent::DeckReshuffled);
    }
    ctx.core.deck.pop()
}

fn detach(core: &mut GameStateCore, card: CardId) -> bool {
    for p in &mut core.players {
        if let Some(i) = p.hand.iter().position(|c| *c == card) {
            p.hand.remove(i);
            return true;
        }
        if let Some(i) = p.equips.iter().position(|c| *c == card) {
            p.equips.remove(i);
            return true;
        }
        if let Some(i) = p.judge.iter().position(|c| *c == card) {
            p.judge.remove(i);
            return true;
        }
    }
    if let Some(i) = core.deck.iter().position(|c| *c == card) {
        core.deck.remove(i);
        return true;
    }
    if let Some(i) = core.discard.iter().position(|c| *c == card) {
        core.discard.remove(i);
        return true;
    }
    for pile in core.vm_aux.piles.values_mut() {
        if let Some(i) = pile.iter().position(|c| *c == card) {
            pile.remove(i);
            return true;
        }
    }
    false
}

pub(crate) fn discard_some(ctx: &mut Ctx<'_>, seat: PlayerId, cards: Vec<CardId>) {
    if cards.is_empty() {
        return;
    }
    ctx.core.discard.extend(cards.iter().copied());
    ctx.out.push(LogEvent::CardDiscarded {
        player: seat,
        cards: cards.clone(),
    });
    ctx.emit(EventData::CardsDiscarded {
        player: seat,
        cards,
    });
}

fn give_pile_to_hand(ctx: &mut Ctx<'_>, seat: PlayerId, key: &(PlayerId, String)) {
    let cards = ctx.core.vm_aux.piles.remove(key).unwrap_or_default();
    if cards.is_empty() {
        return;
    }
    ctx.core.player_mut(seat).hand.extend(cards.iter().copied());
    ctx.emit(EventData::CardsMoved {
        cards: cards.clone(),
        from: seat,
        to: seat,
    });
}

fn push_pile(ctx: &mut Ctx<'_>, seat: PlayerId, name: &str, card: CardId) {
    ctx.core
        .vm_aux
        .piles
        .entry((seat, name.to_string()))
        .or_default()
        .push(card);
}

/// Halted 后落账。`executing` 是正在结算的技能名（牌堆键）。
pub fn commit_effect(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId, e: EffectRequest) {
    let seats = ctx.core.players.len() as u8;
    match e {
        EffectRequest::RecoverHp { player, amount, .. } => {
            if let Some(p) = seat_of(player, seats) {
                ctx.recover(p, amount, None);
            }
        }
        EffectRequest::DrawCards { player, count } => {
            // R05（C 批）：VM 摸牌效果走真实 R 帧（后续步不读所抽牌；
            // 需要结果的 VM 技能经后续查询原语读取）。
            if let Some(p) = seat_of(player, seats) {
                ctx.start_draw(p, count);
            }
        }
        EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => {
            let Some(t) = seat_of(target, seats) else {
                return;
            };
            let src = seat_of(source, seats);
            if let Some(s) = src {
                let key = (s, "lifetime".to_string());
                *ctx.core.vm_aux.flags.entry(key).or_insert(0) += amount;
            }
            if is_lose_hp {
                ctx.emit(EventData::LoseHp { player: t, amount });
            } else {
                let nature = match nature {
                    1 => DamageNature::Fire,
                    2 => DamageNature::Thunder,
                    _ => DamageNature::Normal,
                };
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    src, t, amount as i32, nature, None,
                )));
            }
        }
        EffectRequest::ChangeMaxHp { player, delta } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let pl = ctx.core.player_mut(p);
            let next = pl.max_hp as i32 + delta;
            if next <= 0 {
                pl.max_hp = 1;
                pl.hp = pl.hp.min(1);
            } else {
                pl.max_hp = next as u32;
                if pl.hp > pl.max_hp as i32 {
                    pl.hp = pl.max_hp as i32;
                }
            }
            if delta < 0 {
                ctx.out.push(LogEvent::MaxHpLost {
                    player: p,
                    amount: delta.unsigned_abs(),
                });
            }
        }
        EffectRequest::AddMark {
            player,
            mark,
            delta,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let e = ctx.core.vm_aux.marks.entry((p, mark.raw())).or_insert(0);
            *e += delta;
            if *e == 0 {
                ctx.core.vm_aux.marks.remove(&(p, mark.raw()));
            }
        }
        EffectRequest::RemoveMark {
            player,
            mark,
            amount,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let key = (p, mark.raw());
            if amount == 0 {
                ctx.core.vm_aux.marks.remove(&key);
            } else if let Some(v) = ctx.core.vm_aux.marks.get_mut(&key) {
                *v -= amount as i32;
                if *v <= 0 {
                    ctx.core.vm_aux.marks.remove(&key);
                }
            }
        }
        EffectRequest::DiscardFromHand { player, count } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let n = count as usize;
            let cards: Vec<_> = ctx.core.player(p).hand.iter().copied().take(n).collect();
            for c in &cards {
                ctx.take_hand_card(p, *c);
            }
            discard_some(ctx, p, cards);
        }
        EffectRequest::DiscardFromHandEquip { player, count } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let mut left = count as usize;
            let mut cards = Vec::new();
            while left > 0 {
                if let Some(c) = ctx.core.player(p).hand.first().copied() {
                    ctx.take_hand_card(p, c);
                    cards.push(c);
                    left -= 1;
                } else if let Some(c) = ctx.core.player(p).equips.first().copied() {
                    ctx.take_owned_card(p, c);
                    cards.push(c);
                    left -= 1;
                } else {
                    break;
                }
            }
            discard_some(ctx, p, cards);
        }
        EffectRequest::AddSkill { player, skill_def } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let raw = skill_def.raw();
            if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
                let already_had = ctx.core.has_skill(p, def);
                ctx.ensure_skill(
                    (p, def),
                    SkillInstance {
                        owner: p,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
                if !already_had {
                    let toggled = EventData::SkillToggled {
                        player: p,
                        skill: def,
                        enabled: true,
                        temporary: false,
                    };
                    let qctx = ctx.query();
                    let has_listener = ctx.core.skills.iter().any(|((owner, sdef), inst)| {
                        let def_entry = &ctx.catalog.skills[*sdef as usize];
                        inst.enabled
                            && ctx.core.player(*owner).alive
                            && def_entry.trigger.as_ref().is_some_and(|m| {
                                m.event == crate::event::EventKind::SkillToggled
                                    || m.extra_events
                                        .contains(&crate::event::EventKind::SkillToggled)
                            })
                            && def_entry.behavior.filter_in_window(
                                &qctx,
                                *owner,
                                &toggled,
                                crate::event::EventKind::SkillToggled,
                            )
                    });
                    if has_listener {
                        ctx.emit(toggled);
                    }
                }
                return;
            }
            ctx.core.vm_aux.flags.insert((p, format!("skill:{raw}")), 1);
        }
        EffectRequest::AddSkillUntilTurnEnd { player, skill_def } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let raw = skill_def.raw();
            if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
                if !ctx.core.has_skill(p, def) {
                    ctx.add_skill_until_turn_end(p, def);
                }
                return;
            }
            ctx.core.vm_aux.flags.insert((p, format!("skill:{raw}")), 1);
            ctx.core
                .vm_aux
                .flags
                .insert((p, format!("skill_turn:{raw}")), 1);
        }
        EffectRequest::RemoveSkillByDef { player, skill_def } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let raw = skill_def.raw();
            if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
                ctx.remove_skill(p, def);
            }
            ctx.core.vm_aux.flags.remove(&(p, format!("skill:{raw}")));
            ctx.core
                .vm_aux
                .flags
                .remove(&(p, format!("skill_turn:{raw}")));
        }
        EffectRequest::RemoveSkill { skill } => {
            if let Some(def) = resolve_skill_def(ctx.catalog, skill.raw()) {
                let owners: Vec<PlayerId> = ctx
                    .core
                    .skills
                    .keys()
                    .filter(|(_, d)| *d == def)
                    .map(|(o, _)| *o)
                    .collect();
                for owner in owners {
                    ctx.remove_skill(owner, def);
                }
            }
        }
        EffectRequest::GainTopCardToSkillPile { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(card) = pop_deck(ctx) {
                push_pile(ctx, p, executing, card);
            }
        }
        EffectRequest::PlaceTopDeckToExpansion {
            player,
            pile_def,
            count,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let name = pile_name(ctx.catalog, executing, pile_def);
            for _ in 0..count {
                let Some(card) = pop_deck(ctx) else { break };
                push_pile(ctx, p, &name, card);
            }
        }
        EffectRequest::GainWholeSkillPile {
            player, pile_def, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let name = pile_name(ctx.catalog, executing, pile_def);
            give_pile_to_hand(ctx, p, &(p, name));
        }
        EffectRequest::GainAllSkillPileCards { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let keys: Vec<_> = ctx
                .core
                .vm_aux
                .piles
                .keys()
                .filter(|(s, _)| *s == p)
                .cloned()
                .collect();
            for k in keys {
                give_pile_to_hand(ctx, p, &k);
            }
        }
        EffectRequest::GainFromSkillPile { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let key = (p, executing.to_string());
            let card = ctx.core.vm_aux.piles.get_mut(&key).and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            });
            if let Some(card) = card {
                ctx.core.player_mut(p).hand.push(card);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![card],
                    from: p,
                    to: p,
                });
            }
        }
        EffectRequest::GainOtherSkillPileCards {
            player,
            target,
            skill_def,
            ..
        } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            let name = pile_name(ctx.catalog, executing, skill_def);
            let cards = ctx.core.vm_aux.piles.remove(&(t, name)).unwrap_or_default();
            if cards.is_empty() {
                return;
            }
            ctx.core.player_mut(p).hand.extend(cards.iter().copied());
            ctx.emit(EventData::CardsMoved {
                cards,
                from: t,
                to: p,
            });
        }
        EffectRequest::DiscardSkillPile {
            player, pile_def, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let name = pile_name(ctx.catalog, executing, pile_def);
            let cards = ctx.core.vm_aux.piles.remove(&(p, name)).unwrap_or_default();
            discard_some(ctx, p, cards);
        }
        EffectRequest::StoreEventCardInSkillPile { player, card, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if card.raw() == 0 {
                return;
            }
            let id = card.raw();
            if detach(ctx.core, id) {
                push_pile(ctx, p, executing, id);
            }
        }
        EffectRequest::DiscardAllOwnJudge { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let cards = std::mem::take(&mut ctx.core.player_mut(p).judge);
            discard_some(ctx, p, cards);
        }
        EffectRequest::StorageSet {
            skill: _,
            key,
            value,
        } => {
            let scoped = (owner, storage_key(executing, key));
            if value == 0 {
                ctx.core.vm_aux.flags.remove(&scoped);
            } else {
                ctx.core.vm_aux.flags.insert(scoped, value);
            }
            let k = format!("{executing}#{key}");
            if value == 0 {
                ctx.core.vm_aux.storage.remove(&k);
            } else {
                ctx.core.vm_aux.storage.insert(k, value);
            }
        }
        EffectRequest::SkipCurrentPhase { player } => {
            if let Some(p) = seat_of(player, seats) {
                if p == ctx.core.turn.current {
                    match ctx.core.turn.phase {
                        Phase::Draw => ctx.core.turn.skip_draw_phase = true,
                        Phase::Play => ctx.core.turn.skip_play_phase = true,
                        Phase::Discard => {
                            ctx.core.player_mut(p).skip_discard_phase = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        EffectRequest::PreventDamage { .. } => {
            if let Ok(d) = ctx.current_damage() {
                let id = ctx.current_frame_id();
                let _ = id;
                if d.amount > 0 {
                    ctx.current_damage_mut().prevented = true;
                }
            }
        }
        EffectRequest::ModifyCurrentDamage {
            player: _, delta, ..
        } => {
            if let Ok(d) = ctx.current_damage() {
                if d.amount > 0 && delta != 0 {
                    let Some(next) = d.amount.checked_add(delta) else {
                        ctx.res.fault = Some(crate::machine::VmFault {
                            skill: "modify_current_damage".into(),
                            owner: 0,
                            event: "Damage".into(),
                            kind: "DamageAmountOverflow".into(),
                            detail: "canonical i32 overflow".into(),
                        });
                        return;
                    };
                    ctx.current_damage_mut().amount = next;
                }
            }
        }
        EffectRequest::DiscardHandEquipCount { player, count, .. } => {
            let count = count.max(1);
            if let (Some(p), false) = (seat_of(player, seats), count == 0) {
                let mut left = count as usize;
                let mut cards = Vec::new();
                while left > 0 {
                    let pick = ctx
                        .core
                        .player(p)
                        .hand
                        .first()
                        .copied()
                        .or_else(|| ctx.core.player(p).equips.first().copied());
                    let Some(c) = pick else { break };
                    if ctx.core.player(p).hand.first() == Some(&c) {
                        ctx.take_hand_card(p, c);
                    } else {
                        ctx.take_owned_card(p, c);
                    }
                    cards.push(c);
                    left -= 1;
                }
                discard_some(ctx, p, cards);
            }
        }
        EffectRequest::TurnOver { player } => {
            if let Some(p) = seat_of(player, seats) {
                ctx.emit(EventData::TurnOverBefore {
                    player: p,
                    cancelled: false,
                });
            }
        }
        EffectRequest::EnterDying { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.core.player_mut(p).hp = ctx.core.player(p).hp.min(0);
            ctx.core.player_mut(p).dying = true;
            ctx.out.push(LogEvent::DyingEntered { player: p });
            ctx.emit(EventData::Dying {
                player: p,
                source: None,
            });
        }
        EffectRequest::KillPlayer { player, killer } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let src = seat_of(killer, seats);
            ctx.core.player_mut(p).hp = 0;
            ctx.core.player_mut(p).alive = false;
            ctx.core.player_mut(p).dying = false;
            ctx.out.push(LogEvent::PlayerDied {
                player: p,
                killer: src,
                presentation: None,
            });
            ctx.emit(EventData::Death {
                player: p,
                killer: src,
            });
        }
        EffectRequest::TakeFromHand { from, to } => {
            let (Some(a), Some(b)) = (seat_of(from, seats), seat_of(to, seats)) else {
                return;
            };
            if a == b {
                return;
            }
            if let Some(card) = ctx.core.player(a).hand.first().copied() {
                ctx.give_hand_card(a, b, card);
            }
        }
        EffectRequest::MoveCards { cards, to, .. } => {
            let kind = (to.raw() & 0xff) as u8;
            let owner_raw = to.raw() >> 8;
            let owner = if owner_raw == 0 {
                None
            } else {
                seat_of(rule_api::PlayerId::new(owner_raw), seats)
            };
            for c in cards {
                if c.raw() == 0 || !detach(ctx.core, c.raw()) {
                    continue;
                }
                let id = c.raw();
                match kind {
                    1 => ctx.core.deck.push(id),
                    2 => {
                        ctx.core.discard.push(id);
                        ctx.out.push(LogEvent::CardDiscarded {
                            player: owner.unwrap_or(0),
                            cards: vec![id],
                        });
                    }
                    3 => {
                        let s = owner.unwrap_or(0);
                        ctx.core.player_mut(s).hand.push(id);
                    }
                    4 => {
                        let s = owner.unwrap_or(0);
                        ctx.core.player_mut(s).equips.push(id);
                    }
                    5 => {
                        let s = owner.unwrap_or(0);
                        ctx.core.player_mut(s).judge.push(id);
                    }
                    7 => push_pile(ctx, owner.unwrap_or(0), "removed", id),
                    _ => push_pile(ctx, owner.unwrap_or(0), executing, id),
                }
            }
        }
        EffectRequest::RevealCards { cards, to } => {
            let shown: Vec<_> = cards
                .into_iter()
                .map(|c| c.raw())
                .filter(|c| *c != 0)
                .collect();
            let who: Vec<_> = to.into_iter().filter_map(|p| seat_of(p, seats)).collect();
            if !shown.is_empty() {
                ctx.out.push(LogEvent::CardsRevealed {
                    cards: shown,
                    to: who,
                });
            }
        }
        EffectRequest::RevealHand { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let shown = ctx.core.player(p).hand.clone();
            if !shown.is_empty() {
                ctx.out.push(LogEvent::CardsRevealed {
                    cards: shown,
                    to: Vec::new(),
                });
            }
        }
        EffectRequest::RevealHandToPlayer { player, observer } => {
            let (Some(p), Some(obs)) = (seat_of(player, seats), seat_of(observer, seats)) else {
                return;
            };
            let shown = ctx.core.player(p).hand.clone();
            if !shown.is_empty() {
                ctx.out.push(LogEvent::CardsRevealed {
                    cards: shown,
                    to: vec![obs],
                });
            }
        }
        EffectRequest::GainCardByKind { player, kind } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let found = ctx
                .core
                .deck
                .iter()
                .rposition(|c| kind_tag(ctx.catalog.card_kind(*c)) == kind)
                .map(|i| ctx.core.deck.remove(i))
                .or_else(|| {
                    ctx.core
                        .discard
                        .iter()
                        .rposition(|c| kind_tag(ctx.catalog.card_kind(*c)) == kind)
                        .map(|i| ctx.core.discard.remove(i))
                });
            if let Some(card) = found {
                ctx.core.player_mut(p).hand.push(card);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: p,
                    cards: vec![card],
                });
            }
        }
        EffectRequest::GainDiscardCardByKind { player, kind } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(i) = ctx
                .core
                .discard
                .iter()
                .rposition(|c| kind_tag(ctx.catalog.card_kind(*c)) == kind)
            {
                let card = ctx.core.discard.remove(i);
                ctx.core.player_mut(p).hand.push(card);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: p,
                    cards: vec![card],
                });
            }
        }
        EffectRequest::GainEquipmentFromDrawPile { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(i) = ctx
                .core
                .deck
                .iter()
                .rposition(|c| ctx.catalog.card_kind(*c).is_equip())
            {
                let card = ctx.core.deck.remove(i);
                ctx.core.player_mut(p).hand.push(card);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: p,
                    cards: vec![card],
                });
            }
        }
        EffectRequest::GiveAllHand { player, target } => {
            let (Some(a), Some(b)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if a == b || !ctx.core.player(a).alive || !ctx.core.player(b).alive {
                return;
            }
            let cards = ctx.core.player(a).hand.clone();
            ctx.give_hand_cards(a, b, &cards);
        }
        EffectRequest::DiscardAllTargetEquipment { actor, target } => {
            let (Some(a), Some(t)) = (seat_of(actor, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(a).alive || !ctx.core.player(t).alive {
                return;
            }
            let cards = ctx.core.player(t).equips.clone();
            for c in &cards {
                ctx.take_owned_card(t, *c);
            }
            discard_some(ctx, t, cards);
        }
        EffectRequest::GainEventCards { player, cards } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let mut gained = Vec::new();
            for c in cards {
                let id = c.raw();
                if id != 0 {
                    if let Some(pos) = ctx.core.discard.iter().position(|x| *x == id) {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(p).hand.push(id);
                        gained.push(id);
                    } else if let Some(pos) = ctx.core.table.iter().position(|x| *x == id) {
                        ctx.core.table.remove(pos);
                        ctx.core.player_mut(p).hand.push(id);
                        gained.push(id);
                    }
                }
            }
            if !gained.is_empty() {
                ctx.out.push(LogEvent::CardsDrawn {
                    player: p,
                    cards: gained,
                });
            }
        }
        EffectRequest::GainCardIfDiscard { player, card } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let id = card.raw();
            if id != 0 && ctx.core.player(p).alive {
                if let Some(pos) = ctx.core.discard.iter().position(|x| *x == id) {
                    ctx.core.discard.remove(pos);
                    ctx.core.player_mut(p).hand.push(id);
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: p,
                        cards: vec![id],
                    });
                }
            }
        }
        EffectRequest::IncreaseCurrentDrawCount { player, count } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if p == ctx.core.turn.current && ctx.core.turn.phase == Phase::Draw {
                *ctx.core
                    .vm_aux
                    .flags
                    .entry((p, "draw_phase_bonus".to_string()))
                    .or_insert(0) += count;
            }
        }
        EffectRequest::SkipNextPlayPhase { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if p == ctx.core.turn.current && ctx.core.player(p).alive {
                ctx.core.turn.skip_play_phase = true;
            }
        }
        EffectRequest::SkipNextDiscardPhase { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if ctx.core.player(p).alive {
                ctx.core.player_mut(p).skip_discard_phase = true;
            }
        }
        EffectRequest::SkipNextJudgePhase { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if ctx.core.player(p).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, "skip_judge".to_string()), 1);
            }
        }
        EffectRequest::SkipJudgeAndDrawPhases { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if p == ctx.core.turn.current && ctx.core.player(p).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, "skip_judge".to_string()), 1);
                ctx.core.turn.skip_draw_phase = true;
            }
        }
        EffectRequest::SkipRemainingPhases {
            player,
            phases_mask,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            if phases_mask & 0b0100 != 0 {
                ctx.core.turn.skip_draw_phase = true;
            }
            if phases_mask & 0b1000 != 0 {
                ctx.core.turn.skip_play_phase = true;
            }
            if phases_mask & 0b1_0000 != 0 {
                ctx.core.player_mut(p).skip_discard_phase = true;
            }
        }
        EffectRequest::QueueExtraPhases {
            player,
            phases_mask,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            for tag in 1u8..=5u8 {
                if phases_mask & (1 << tag) != 0 {
                    ctx.core.turn.extra_phases.push(tag - 1);
                }
            }
        }
        EffectRequest::GainTargetEquipment {
            player,
            target,
            slot,
        } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive || !ctx.core.player(t).alive {
                return;
            }
            let want = match slot {
                1 => crate::catalog::EquipSlot::Weapon,
                2 => crate::catalog::EquipSlot::Armor,
                3 => crate::catalog::EquipSlot::HorsePlus,
                4 => crate::catalog::EquipSlot::HorseMinus,
                5 => crate::catalog::EquipSlot::Treasure,
                _ => return,
            };
            let matching: Vec<CardId> = ctx
                .core
                .player(t)
                .equips
                .iter()
                .copied()
                .filter(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(want))
                .collect();
            for c in matching {
                ctx.take_owned_card(t, c);
                ctx.core.player_mut(p).hand.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: t,
                    to: p,
                });
            }
        }
        EffectRequest::RefundCurrentShaUse { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if p == ctx.core.turn.current {
                ctx.core.turn.sha_used_play_phase = false;
            }
        }
        EffectRequest::SetTurnState { current, phase } => {
            let Some(p) = seat_of(current, seats) else {
                return;
            };
            let phase = Phase::ALL[phase as usize];
            ctx.core.turn.current = p;
            ctx.core.turn.phase = phase;
            ctx.out.push(LogEvent::PhaseChanged {
                player: p,
                phase,
                begin: true,
            });
        }
        _ => {}
    }
}
