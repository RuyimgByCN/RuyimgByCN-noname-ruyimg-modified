//! VM 查询（QueryRequest）在原生状态上的只读求值。

use super::*;

pub(super) fn suit_of(catalog: &Catalog, card: u32) -> Option<Suit> {
    catalog.cards.get(card as usize).map(|c| c.suit)
}

pub(super) fn suit_tag(s: Suit) -> u8 {
    match s {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn count_suit(cards: &[CardId], catalog: &Catalog, suit: u8) -> i32 {
    cards
        .iter()
        .filter(|c| suit_of(catalog, **c).is_some_and(|s| suit_tag(s) == suit))
        .count() as i32
}

pub(super) fn locate(core: &GameStateCore, card: u32) -> i32 {
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
