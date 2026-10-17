//! T8 余项：VM 效果与查询的真实读写。
//!
//! 未列入本文件的效果（具名结算器、选择链、用牌管线）在入队前拒绝，
//! 避免记成已支持的空成功。区域号由 `QueryRequest::Zone` 打包
//! （`rule_player << 8 | kind`），`MoveCards` 按同一布局拆开。
//!
//! 登记差异：终身伤害只累计本桥提交的 `DealDamage`（写入 `vm_aux.flags`
//! 键 `lifetime`），内核伤害仍只在 `turn_damage`。规则牌 id 0 与哨兵
//! 冲突，效果里的 0 视为无牌。`SkipCurrentPhase` / `UseCard` 无内核
//! 落点，拒绝。

use rule_api::{EffectRequest, HostTrap, QueryRequest, QueryResponse, RelationTag};

use crate::catalog::{CardKind, Catalog, Faction, Suit};
use crate::event::{DamageNature, EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

fn reject(name: &str) -> HostTrap {
    HostTrap::InvalidRequest {
        message: format!("rvm_bridge 未映射: {name}"),
    }
}

fn seat_of(p: rule_api::PlayerId, seats: u8) -> Option<PlayerId> {
    if !p.is_some() {
        return None;
    }
    let raw = p.raw() - 1;
    let s = u8::try_from(raw).ok()?;
    (s < seats).then_some(s)
}

fn kind_tag(kind: CardKind) -> u8 {
    CardKind::ALL
        .iter()
        .position(|k| *k == kind)
        .unwrap_or(255) as u8
}

fn skill_label(catalog: &Catalog, def: u32) -> Option<String> {
    let id = u16::try_from(def).ok()?;
    ((id as usize) < catalog.skills.len()).then(|| catalog.skill_name(id).to_string())
}

fn pile_name(catalog: &Catalog, executing: &str, pile_def: u32) -> String {
    skill_label(catalog, pile_def).unwrap_or_else(|| executing.to_string())
}

/// 入队前校验。通过的变体必须能编码、能落账。
pub fn accept_effect(req: &EffectRequest, seats: u8) -> Result<(), HostTrap> {
    let ok_player = |p: rule_api::PlayerId, what: &str| -> Result<PlayerId, HostTrap> {
        seat_of(p, seats).ok_or_else(|| reject(what))
    };
    match req {
        EffectRequest::RecoverHp { player, amount, .. } => {
            ok_player(*player, "RecoverHp")?;
            if *amount == 0 {
                return Err(reject("RecoverHp 零量"));
            }
        }
        EffectRequest::DrawCards { player, count } => {
            ok_player(*player, "DrawCards")?;
            if *count == 0 {
                return Err(reject("DrawCards 零量"));
            }
        }
        EffectRequest::DealDamage { target, amount, nature, .. } => {
            ok_player(*target, "DealDamage")?;
            if *amount == 0 || *nature > 2 {
                return Err(reject("DealDamage 越界"));
            }
        }
        EffectRequest::ChangeMaxHp { player, delta } => {
            ok_player(*player, "ChangeMaxHp")?;
            if *delta == 0 {
                return Err(reject("ChangeMaxHp 零量"));
            }
        }
        EffectRequest::AddMark { player, .. } | EffectRequest::RemoveMark { player, .. } => {
            ok_player(*player, "Mark")?;
        }
        EffectRequest::DiscardFromHand { player, count }
        | EffectRequest::DiscardFromHandEquip { player, count } => {
            ok_player(*player, "Discard")?;
            if *count == 0 {
                return Err(reject("Discard 零量"));
            }
        }
        EffectRequest::AddSkill { player, .. }
        | EffectRequest::EnterDying { player }
        | EffectRequest::DiscardAllOwnJudge { player, .. }
        | EffectRequest::GainTopCardToSkillPile { player, .. }
        | EffectRequest::GainAllSkillPileCards { player, .. }
        | EffectRequest::GainFromSkillPile { player, .. }
        | EffectRequest::DiscardSkillPile { player, .. }
        | EffectRequest::PlaceTopDeckToExpansion { player, .. }
        | EffectRequest::GainWholeSkillPile { player, .. }
        | EffectRequest::StoreEventCardInSkillPile { player, .. } => {
            ok_player(*player, "player")?;
        }
        EffectRequest::GainOtherSkillPileCards { player, target, .. } => {
            ok_player(*player, "GainOther.player")?;
            ok_player(*target, "GainOther.target")?;
        }
        EffectRequest::RemoveSkill { skill } => {
            if !skill.is_some() {
                return Err(reject("RemoveSkill.NONE"));
            }
        }
        EffectRequest::KillPlayer { player, .. } => {
            ok_player(*player, "KillPlayer")?;
        }
        EffectRequest::TakeFromHand { from, to } => {
            ok_player(*from, "TakeFromHand.from")?;
            ok_player(*to, "TakeFromHand.to")?;
        }
        EffectRequest::MoveCards { cards, to, .. } => {
            if cards.is_empty() || to.raw() == 0 {
                return Err(reject("MoveCards 空"));
            }
        }
        EffectRequest::RevealCards { cards, .. } => {
            if cards.is_empty() {
                return Err(reject("RevealCards 空"));
            }
        }
        EffectRequest::SetTurnState { current, phase } => {
            ok_player(*current, "SetTurnState")?;
            if *phase > 4 {
                return Err(reject("SetTurnState phase"));
            }
        }
        other => {
            let name = format!("{other:?}");
            return Err(reject(name.split([' ', '(', '{']).next().unwrap_or("?")));
        }
    }
    Ok(())
}

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

fn discard_some(ctx: &mut Ctx<'_>, seat: PlayerId, cards: Vec<CardId>) {
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
pub fn commit_effect(ctx: &mut Ctx<'_>, executing: &str, e: EffectRequest) {
    let seats = ctx.core.players.len() as u8;
    match e {
        EffectRequest::RecoverHp { player, amount, .. } => {
            if let Some(p) = seat_of(player, seats) {
                ctx.recover(p, amount, None);
            }
        }
        EffectRequest::DrawCards { player, count } => {
            if let Some(p) = seat_of(player, seats) {
                ctx.draw_cards(p, count as usize);
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
                    src, t, amount, nature, None,
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
        EffectRequest::AddMark { player, mark, delta } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let e = ctx.core.vm_aux.marks.entry((p, mark.raw())).or_insert(0);
            *e += delta;
            if *e == 0 {
                ctx.core.vm_aux.marks.remove(&(p, mark.raw()));
            }
        }
        EffectRequest::RemoveMark { player, mark, amount } => {
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
            if let Ok(def) = u16::try_from(raw) {
                if (def as usize) < ctx.catalog.skills.len() {
                    ctx.core.skills.entry((p, def)).or_insert(SkillInstance {
                        owner: p,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    });
                    return;
                }
            }
            ctx.core.vm_aux.flags.insert((p, format!("skill:{raw}")), 1);
        }
        EffectRequest::RemoveSkill { skill } => {
            if let Ok(def) = u16::try_from(skill.raw()) {
                ctx.core.skills.retain(|(_, d), _| *d != def);
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
        EffectRequest::PlaceTopDeckToExpansion { player, pile_def, count, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let name = pile_name(ctx.catalog, executing, pile_def);
            for _ in 0..count {
                let Some(card) = pop_deck(ctx) else { break };
                push_pile(ctx, p, &name, card);
            }
        }
        EffectRequest::GainWholeSkillPile { player, pile_def, .. } => {
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
        EffectRequest::DiscardSkillPile { player, pile_def, .. } => {
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
            let shown: Vec<_> = cards.into_iter().map(|c| c.raw()).filter(|c| *c != 0).collect();
            let who: Vec<_> = to.into_iter().filter_map(|p| seat_of(p, seats)).collect();
            if !shown.is_empty() {
                ctx.out.push(LogEvent::CardsRevealed { cards: shown, to: who });
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

fn suit_of(catalog: &Catalog, card: u32) -> Option<Suit> {
    catalog.cards.get(card as usize).map(|c| c.suit)
}

fn suit_tag(s: Suit) -> u8 {
    match s {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

fn count_suit(cards: &[CardId], catalog: &Catalog, suit: u8) -> i32 {
    cards
        .iter()
        .filter(|c| suit_of(catalog, **c).is_some_and(|s| suit_tag(s) == suit))
        .count() as i32
}

fn locate(core: &GameStateCore, card: u32) -> i32 {
    if core.deck.contains(&card) {
        return 1;
    }
    if core.discard.contains(&card) {
        return 2;
    }
    for p in &core.players {
        if p.hand.contains(&card) {
            return 3;
        }
        if p.equips.contains(&card) {
            return 4;
        }
        if p.judge.contains(&card) {
            return 5;
        }
    }
    if core.vm_aux.piles.values().any(|v| v.contains(&card)) {
        return 8;
    }
    0
}

/// 41 个查询变体都读当前局面。
pub fn run_query(
    core: &GameStateCore,
    catalog: &Catalog,
    seats: u8,
    req: QueryRequest,
) -> Result<QueryResponse, HostTrap> {
    let seat = |p: rule_api::PlayerId, what: &str| -> Result<PlayerId, HostTrap> {
        seat_of(p, seats).ok_or_else(|| reject(what))
    };
    Ok(match req {
        QueryRequest::Distance { from, to } => {
            let a = seat(from, "Distance")?;
            let b = seat(to, "Distance")?;
            QueryResponse::Int(core.computed_distance(catalog, a, b) as i32)
        }
        QueryRequest::AttackRange { player } => {
            let p = seat(player, "AttackRange")?;
            QueryResponse::Int(core.attack_range(catalog, p) as i32)
        }
        QueryRequest::CardUsable { player, card } => {
            let p = seat(player, "CardUsable")?;
            let pl = core.player(p);
            let usable = pl.alive
                && core.turn.current == p
                && core.turn.phase == Phase::Play
                && card.raw() != 0
                && pl.hand.contains(&card.raw());
            QueryResponse::Bool(usable)
        }
        QueryRequest::TargetLegal { user, card, target } => {
            let u = seat(user, "TargetLegal.user")?;
            let t = seat(target, "TargetLegal.target")?;
            let legal = core.player(u).alive
                && core.player(t).alive
                && u != t
                && card.raw() != 0
                && catalog.cards.get(card.raw() as usize).is_some_and(|def| {
                    if def.kind.is_sha_like() {
                        core.computed_distance(catalog, u, t) <= core.attack_range(catalog, u)
                    } else {
                        true
                    }
                });
            QueryResponse::Bool(legal)
        }
        QueryRequest::HandLimit { player } => {
            let p = seat(player, "HandLimit")?;
            QueryResponse::Int(core.hand_limit(catalog, p) as i32)
        }
        QueryRequest::SkillUsesRemaining { skill } => {
            let left = u16::try_from(skill.raw()).ok().and_then(|def| {
                core.skills.iter().find_map(|((_, d), inst)| {
                    (*d == def).then_some(if inst.once_used { 0 } else { 1 })
                })
            });
            QueryResponse::Int(left.unwrap_or(0))
        }
        QueryRequest::CardAttribute { card, attr } => {
            let v = catalog.cards.get(card.raw() as usize).map(|def| match attr {
                1 => suit_tag(def.suit) as i32,
                2 => def.rank as i32,
                _ => kind_tag(def.kind) as i32,
            });
            QueryResponse::Int(v.unwrap_or(0))
        }
        QueryRequest::Relation { a, b } => {
            let a = seat(a, "Relation")?;
            let b = seat(b, "Relation")?;
            let tag = if a == b {
                RelationTag::Self_
            } else {
                match (core.effective_faction(catalog, a), core.effective_faction(catalog, b)) {
                    (Some(x), Some(y)) if x == y => RelationTag::Ally,
                    (Some(_), Some(_)) => RelationTag::Enemy,
                    _ => RelationTag::Unknown,
                }
            };
            QueryResponse::Relation(tag)
        }
        QueryRequest::Hp { player } => {
            QueryResponse::Int(core.player(seat(player, "Hp")?).hp)
        }
        QueryRequest::HandCount { player } => {
            QueryResponse::Int(core.player(seat(player, "HandCount")?).hand.len() as i32)
        }
        QueryRequest::EquippedCount { player } => {
            QueryResponse::Int(core.player(seat(player, "Equipped")?).equips.len() as i32)
        }
        QueryRequest::EmptyEquipSlots { player } => {
            let p = seat(player, "EmptyEquip")?;
            let mut filled = [false; 5];
            for c in &core.player(p).equips {
                if let Some(slot) = catalog.card_slot_of(core, *c) {
                    let i = match slot {
                        crate::catalog::EquipSlot::Weapon => 0,
                        crate::catalog::EquipSlot::Armor => 1,
                        crate::catalog::EquipSlot::HorsePlus => 2,
                        crate::catalog::EquipSlot::HorseMinus => 3,
                        crate::catalog::EquipSlot::Treasure => 4,
                    };
                    filled[i] = true;
                }
            }
            QueryResponse::Int(filled.iter().filter(|s| !**s).count() as i32)
        }
        QueryRequest::MaxHp { player } => {
            QueryResponse::Int(core.player(seat(player, "MaxHp")?).max_hp as i32)
        }
        QueryRequest::Zone { player, kind } => {
            let packed = (player.raw() << 8) | kind as u32;
            QueryResponse::Int(packed as i32)
        }
        QueryRequest::CardZone { card } => QueryResponse::Int(locate(core, card.raw())),
        QueryRequest::MarkCount { player, mark } => {
            let p = seat(player, "MarkCount")?;
            QueryResponse::Int(*core.vm_aux.marks.get(&(p, mark)).unwrap_or(&0))
        }
        QueryRequest::AliveCount => {
            QueryResponse::Int(core.players.iter().filter(|p| p.alive).count() as i32)
        }
        QueryRequest::CardActionCount { player } => {
            let _ = seat(player, "CardAction")?;
            QueryResponse::Int(core.turn.cards_used_play_phase as i32)
        }
        QueryRequest::AliveGroupCount => {
            let mut seen = Vec::new();
            for (i, p) in core.players.iter().enumerate() {
                if !p.alive {
                    continue;
                }
                if let Some(f) = core.effective_faction(catalog, i as u8) {
                    if !seen.contains(&f) {
                        seen.push(f);
                    }
                }
            }
            QueryResponse::Int(seen.len() as i32)
        }
        QueryRequest::UniqueMaxCurrentHp { player } => {
            let p = seat(player, "UniqueMax")?;
            let me = core.player(p).hp;
            let unique = core.player(p).alive
                && core.players.iter().enumerate().all(|(i, o)| {
                    i as u8 == p || !o.alive || o.hp < me
                });
            QueryResponse::Bool(unique)
        }
        QueryRequest::UniqueMaxCurrentHpAt { player, value } => {
            let p = seat(player, "UniqueMaxAt")?;
            let unique = core.player(p).alive
                && core.players.iter().enumerate().all(|(i, o)| {
                    i as u8 == p || !o.alive || o.hp < value
                });
            QueryResponse::Bool(unique)
        }
        QueryRequest::LifetimeDamageDealt { player } => {
            let p = seat(player, "Lifetime")?;
            QueryResponse::Int(
                core.vm_aux
                    .flags
                    .get(&(p, "lifetime".to_string()))
                    .copied()
                    .unwrap_or(0) as i32,
            )
        }
        QueryRequest::PublicCardKindCount { kind } => {
            let n = core
                .deck
                .iter()
                .chain(core.discard.iter())
                .filter(|c| kind_tag(catalog.card_kind(**c)) == kind)
                .count();
            QueryResponse::Int(n as i32)
        }
        QueryRequest::RoundNumberCapped { cap } => {
            QueryResponse::Int(core.turn.round.min(cap) as i32)
        }
        QueryRequest::FieldEquipmentKindCount { kind } => {
            let n = core
                .players
                .iter()
                .filter(|p| p.alive)
                .flat_map(|p| p.equips.iter())
                .filter(|c| kind_tag(catalog.card_kind(**c)) == kind)
                .count();
            QueryResponse::Int(n as i32)
        }
        QueryRequest::AliveWoundedOrGuimingWuCount { player } => {
            let p = seat(player, "Guiming")?;
            let mut n = core
                .players
                .iter()
                .filter(|pl| pl.alive && pl.hp > 0 && (pl.hp as u32) < pl.max_hp)
                .count() as i32;
            let lord = core
                .identity_roles()
                .and_then(|r| r.get(p as usize))
                .is_some_and(|r| *r == crate::state::Role::Lord);
            let guiming = core.skills.iter().any(|((o, def), inst)| {
                *o == p && inst.enabled && catalog.skill_name(*def).contains("归命")
            });
            if lord && guiming {
                n += core
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(i, pl)| {
                        pl.alive && core.effective_faction(catalog, *i as u8) == Some(Faction::Wu)
                    })
                    .count() as i32;
            }
            QueryResponse::Int(n)
        }
        QueryRequest::HandEquipSuitCount { player, suit } => {
            let p = seat(player, "HandEquipSuit")?;
            let pl = core.player(p);
            let mut cards = pl.hand.clone();
            cards.extend(pl.equips.iter().copied());
            QueryResponse::Int(count_suit(&cards, catalog, suit))
        }
        QueryRequest::CurrentPhasePlayer => {
            QueryResponse::Int(core.turn.current as i32 + 1)
        }
        QueryRequest::HandSuitCount { player, suit } => {
            let p = seat(player, "HandSuit")?;
            QueryResponse::Int(count_suit(&core.player(p).hand, catalog, suit))
        }
        QueryRequest::HandColorCount { player, color } => {
            let p = seat(player, "HandColor")?;
            let n = core
                .player(p)
                .hand
                .iter()
                .filter(|c| {
                    suit_of(catalog, **c).is_some_and(|s| {
                        let red = matches!(s, Suit::Heart | Suit::Diamond);
                        (color == 1 && red) || (color == 2 && !red)
                    })
                })
                .count();
            QueryResponse::Int(n as i32)
        }
        QueryRequest::SkillContentFlag { player, skill_def, bit } => {
            let p = seat(player, "SkillFlag")?;
            let name = skill_label(catalog, skill_def.raw()).unwrap_or_default();
            let bits = core.vm_aux.flags.get(&(p, name)).copied().unwrap_or(0);
            let on = bit < 32 && (bits & (1 << bit)) != 0;
            QueryResponse::Int(i32::from(on))
        }
        QueryRequest::SkillPileCount { player, skill_def } => {
            let p = seat(player, "SkillPile")?;
            let name = skill_label(catalog, skill_def).unwrap_or_default();
            let n = core
                .vm_aux
                .piles
                .get(&(p, name))
                .map(|c| c.len())
                .unwrap_or(0);
            QueryResponse::SkillPileCount { count: n as i32 }
        }
        QueryRequest::CardsDrawnThisTurn { player } => {
            let p = seat(player, "Drawn")?;
            QueryResponse::Int(core.vm_aux.drawn.get(&p).copied().unwrap_or(0) as i32)
        }
        QueryRequest::DamageDealtThisTurn { player } => {
            let p = seat(player, "TurnDmg")?;
            QueryResponse::Int(core.player(p).turn_damage as i32)
        }
        QueryRequest::HasSkill { player, skill_def } => {
            let p = seat(player, "HasSkill")?;
            let has = u16::try_from(skill_def.raw()).is_ok_and(|d| core.has_skill(p, d));
            QueryResponse::Bool(has)
        }
        QueryRequest::CardNameLength { card, .. } => {
            let n = catalog
                .cards
                .get(card.raw() as usize)
                .map(|c| c.name.chars().count())
                .unwrap_or(0);
            QueryResponse::Int(n as i32)
        }
        QueryRequest::InPlayPhase => QueryResponse::Bool(core.turn.phase == Phase::Play),
        QueryRequest::CardHasTag { card, tag } => {
            let name = skill_label(catalog, tag).unwrap_or_default();
            let hit = core.vm_aux.piles.iter().any(|((_, pile), cards)| {
                pile == &name && cards.contains(&card.raw())
            });
            QueryResponse::Bool(hit)
        }
        QueryRequest::StorageGet { skill, key } => {
            let needle = format!("{}:{key}", skill.raw());
            let from_flag = core
                .vm_aux
                .flags
                .iter()
                .find(|((_, name), _)| name == &needle)
                .map(|(_, v)| *v as i32);
            let from_mark = core
                .vm_aux
                .marks
                .iter()
                .find(|((_, m), _)| *m == key)
                .map(|(_, v)| *v);
            QueryResponse::Int(from_flag.or(from_mark).unwrap_or(0))
        }
        QueryRequest::StorageHas { skill, key } => {
            let needle = format!("{}:{key}", skill.raw());
            let has = core.vm_aux.flags.keys().any(|(_, name)| name == &needle)
                || core.vm_aux.marks.keys().any(|(_, m)| *m == key);
            QueryResponse::Bool(has)
        }
        QueryRequest::CardsUsedThisRound { player } => {
            let _ = seat(player, "UsedRound")?;
            QueryResponse::Int(core.turn.cards_used_play_phase as i32)
        }
    })
}

fn push_u32(w: &mut Vec<u8>, v: u32) {
    w.extend_from_slice(&v.to_le_bytes());
}
fn push_i32(w: &mut Vec<u8>, v: i32) {
    w.extend_from_slice(&v.to_le_bytes());
}

/// 排队效果编码。只覆盖 `accept_effect` 放行的变体。
pub fn enc_effects(w: &mut Vec<u8>, effects: &[EffectRequest]) {
    push_u32(w, effects.len() as u32);
    for e in effects {
        match e {
            EffectRequest::RecoverHp { source, player, amount } => {
                w.push(1);
                push_u32(w, source.raw());
                push_u32(w, player.raw());
                push_u32(w, *amount);
            }
            EffectRequest::DrawCards { player, count } => {
                w.push(2);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::DealDamage {
                source,
                target,
                amount,
                is_lose_hp,
                nature,
            } => {
                w.push(3);
                push_u32(w, source.raw());
                push_u32(w, target.raw());
                push_u32(w, *amount);
                w.push(u8::from(*is_lose_hp));
                w.push(*nature);
            }
            EffectRequest::ChangeMaxHp { player, delta } => {
                w.push(4);
                push_u32(w, player.raw());
                push_i32(w, *delta);
            }
            EffectRequest::AddMark { player, mark, delta } => {
                w.push(5);
                push_u32(w, player.raw());
                push_u32(w, mark.raw());
                push_i32(w, *delta);
            }
            EffectRequest::RemoveMark { player, mark, amount } => {
                w.push(6);
                push_u32(w, player.raw());
                push_u32(w, mark.raw());
                push_u32(w, *amount);
            }
            EffectRequest::DiscardFromHand { player, count } => {
                w.push(7);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::DiscardFromHandEquip { player, count } => {
                w.push(8);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::AddSkill { player, skill_def } => {
                w.push(9);
                push_u32(w, player.raw());
                push_u32(w, skill_def.raw());
            }
            EffectRequest::RemoveSkill { skill } => {
                w.push(10);
                push_u32(w, skill.raw());
            }
            EffectRequest::GainTopCardToSkillPile { player, skill } => {
                w.push(11);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::GainWholeSkillPile {
                player,
                skill,
                pile_def,
            } => {
                w.push(12);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
                push_u32(w, *pile_def);
            }
            EffectRequest::GainAllSkillPileCards { player, skill } => {
                w.push(13);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::GainFromSkillPile { player, skill } => {
                w.push(14);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::DiscardSkillPile {
                player,
                skill,
                pile_def,
            } => {
                w.push(15);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
                push_u32(w, *pile_def);
            }
            EffectRequest::StoreEventCardInSkillPile { player, card, skill } => {
                w.push(16);
                push_u32(w, player.raw());
                push_u32(w, card.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::PlaceTopDeckToExpansion {
                player,
                skill,
                pile_def,
                count,
            } => {
                w.push(17);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
                push_u32(w, *pile_def);
                push_u32(w, *count);
            }
            EffectRequest::EnterDying { player } => {
                w.push(18);
                push_u32(w, player.raw());
            }
            EffectRequest::KillPlayer { player, killer } => {
                w.push(19);
                push_u32(w, player.raw());
                push_u32(w, killer.raw());
            }
            EffectRequest::MoveCards { cards, to, from } => {
                w.push(20);
                push_u32(w, cards.len() as u32);
                for c in cards {
                    push_u32(w, c.raw());
                }
                push_u32(w, to.raw());
                match from {
                    Some(z) => {
                        w.push(1);
                        push_u32(w, z.raw());
                    }
                    None => w.push(0),
                }
            }
            EffectRequest::RevealCards { cards, to } => {
                w.push(21);
                push_u32(w, cards.len() as u32);
                for c in cards {
                    push_u32(w, c.raw());
                }
                push_u32(w, to.len() as u32);
                for p in to {
                    push_u32(w, p.raw());
                }
            }
            EffectRequest::SetTurnState { current, phase } => {
                w.push(22);
                push_u32(w, current.raw());
                w.push(*phase);
            }
            EffectRequest::TakeFromHand { from, to } => {
                w.push(23);
                push_u32(w, from.raw());
                push_u32(w, to.raw());
            }
            EffectRequest::DiscardAllOwnJudge { player, skill } => {
                w.push(24);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::GainOtherSkillPileCards {
                player,
                target,
                skill_def,
                skill,
            } => {
                w.push(25);
                push_u32(w, player.raw());
                push_u32(w, target.raw());
                push_u32(w, *skill_def);
                push_u32(w, skill.raw());
            }
            _ => unreachable!("accept_effect 已拒绝未编码变体"),
        }
    }
}

struct Rd<'a> {
    r: &'a [u8],
}
impl Rd<'_> {
    fn u32(&mut self) -> Option<u32> {
        if self.r.len() < 4 {
            return None;
        }
        let (a, rest) = self.r.split_at(4);
        self.r = rest;
        Some(u32::from_le_bytes(a.try_into().ok()?))
    }
    fn i32(&mut self) -> Option<i32> {
        self.u32().map(|v| v as i32)
    }
    fn u8(&mut self) -> Option<u8> {
        let v = *self.r.first()?;
        self.r = &self.r[1..];
        Some(v)
    }
}

/// 与 `enc_effects` 对偶。未知标签或尾部残留返回 None。
pub fn dec_effects(bytes: &[u8]) -> Option<Vec<EffectRequest>> {
    let mut rd = Rd { r: bytes };
    let n = rd.u32()? as usize;
    let mut out = Vec::new();
    for _ in 0..n {
        let tag = rd.u8()?;
        let e = match tag {
            1 => EffectRequest::RecoverHp {
                source: rule_api::PlayerId::new(rd.u32()?),
                player: rule_api::PlayerId::new(rd.u32()?),
                amount: rd.u32()?,
            },
            2 => EffectRequest::DrawCards {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            3 => EffectRequest::DealDamage {
                source: rule_api::PlayerId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
                amount: rd.u32()?,
                is_lose_hp: rd.u8()? == 1,
                nature: rd.u8()?,
            },
            4 => EffectRequest::ChangeMaxHp {
                player: rule_api::PlayerId::new(rd.u32()?),
                delta: rd.i32()?,
            },
            5 => EffectRequest::AddMark {
                player: rule_api::PlayerId::new(rd.u32()?),
                mark: rule_api::MarkDefId::new(rd.u32()?),
                delta: rd.i32()?,
            },
            6 => EffectRequest::RemoveMark {
                player: rule_api::PlayerId::new(rd.u32()?),
                mark: rule_api::MarkDefId::new(rd.u32()?),
                amount: rd.u32()?,
            },
            7 => EffectRequest::DiscardFromHand {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            8 => EffectRequest::DiscardFromHandEquip {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            9 => EffectRequest::AddSkill {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill_def: rule_api::SkillDefId::new(rd.u32()?),
            },
            10 => EffectRequest::RemoveSkill {
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            11 => EffectRequest::GainTopCardToSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            12 => EffectRequest::GainWholeSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
                pile_def: rd.u32()?,
            },
            13 => EffectRequest::GainAllSkillPileCards {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            14 => EffectRequest::GainFromSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            15 => EffectRequest::DiscardSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
                pile_def: rd.u32()?,
            },
            16 => EffectRequest::StoreEventCardInSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            17 => EffectRequest::PlaceTopDeckToExpansion {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
                pile_def: rd.u32()?,
                count: rd.u32()?,
            },
            18 => EffectRequest::EnterDying {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            19 => EffectRequest::KillPlayer {
                player: rule_api::PlayerId::new(rd.u32()?),
                killer: rule_api::PlayerId::new(rd.u32()?),
            },
            20 => {
                let n = rd.u32()? as usize;
                let mut cards = Vec::with_capacity(n);
                for _ in 0..n {
                    cards.push(rule_api::CardId::new(rd.u32()?));
                }
                let to = rule_api::ZoneId::new(rd.u32()?);
                let from = if rd.u8()? == 1 {
                    Some(rule_api::ZoneId::new(rd.u32()?))
                } else {
                    None
                };
                EffectRequest::MoveCards { cards, to, from }
            }
            21 => {
                let n = rd.u32()? as usize;
                let mut cards = Vec::with_capacity(n);
                for _ in 0..n {
                    cards.push(rule_api::CardId::new(rd.u32()?));
                }
                let m = rd.u32()? as usize;
                let mut to = Vec::with_capacity(m);
                for _ in 0..m {
                    to.push(rule_api::PlayerId::new(rd.u32()?));
                }
                EffectRequest::RevealCards { cards, to }
            }
            22 => EffectRequest::SetTurnState {
                current: rule_api::PlayerId::new(rd.u32()?),
                phase: rd.u8()?,
            },
            23 => EffectRequest::TakeFromHand {
                from: rule_api::PlayerId::new(rd.u32()?),
                to: rule_api::PlayerId::new(rd.u32()?),
            },
            24 => EffectRequest::DiscardAllOwnJudge {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            25 => EffectRequest::GainOtherSkillPileCards {
                player: rule_api::PlayerId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
                skill_def: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            _ => return None,
        };
        out.push(e);
    }
    if rd.r.is_empty() {
        Some(out)
    } else {
        None
    }
}

