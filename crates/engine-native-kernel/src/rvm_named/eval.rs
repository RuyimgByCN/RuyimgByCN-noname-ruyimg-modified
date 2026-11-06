//! `NamedProgram` 与 `Atom` 在原生只读状态上的求值实现。

use super::*;

pub fn eval_program(
    prog: &NamedProgram,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    skill: &str,
    view: &FilterView,
) -> bool {
    prog.groups.iter().any(|g| {
        g.iter().all(|c| {
            let v = eval_atom(&c.atom, ctx, owner, skill, view);
            if c.neg { !v } else { v }
        })
    })
}

fn phase_is(ctx: &QueryCtx<'_>, v: &FilterView, phase: Phase) -> bool {
    match v.phase {
        Some(p) => p == phase,
        None => ctx.state.turn.phase == phase,
    }
}

fn eval_atom(atom: &Atom, ctx: &QueryCtx<'_>, owner: PlayerId, skill: &str, v: &FilterView) -> bool {
    let me = ctx.state.player(owner);
    let alive = me.alive;
    let wounded = me.hp > 0 && (me.hp as u32) < me.max_hp;
    let card_kind = v
        .event_card
        .and_then(|c| ctx.catalog.cards.get(c as usize).map(|d| d.kind));
    let suit = v
        .event_card
        .and_then(|c| ctx.catalog.cards.get(c as usize).map(|d| d.suit));
    let logical = v.use_as.or(card_kind);
    let sha = logical.is_some_and(|k| k.is_sha_like());
    let red = suit.is_some_and(|s| matches!(s, Suit::Heart | Suit::Diamond));
    let actor = v.event_source.or(v.event_player);
    match atom {
        Atom::EvPlayer => v.event_player == Some(owner),
        Atom::EvActor => actor == Some(owner),
        Atom::OwnerAlive => alive,
        Atom::InTargets => {
            if !v.targets.is_empty() {
                v.targets.contains(&owner)
            } else {
                v.event_player == Some(owner)
            }
        }
        Atom::UserOther => {
            let user = if v.kind == 1 {
                v.event_source
            } else {
                v.event_player
            };
            user.is_some_and(|u| u != owner)
        }
        Atom::Wounded => wounded,
        Atom::Healthy => alive && !wounded && me.hp > 0,
        Atom::Linked => me.linked,
        Atom::Unlinked => !me.linked,
        Atom::FaceDown => me.face_down,
        Atom::FaceUp => !me.face_down,
        Atom::HandEmpty => me.hand.is_empty(),
        Atom::HandNonEmpty => !me.hand.is_empty(),
        Atom::HandBelowMax => me.hand.len() < me.max_hp as usize,
        Atom::HandAboveHp => me.hand.len() > me.hp.max(0) as usize,
        Atom::HandAtLeastHp => me.hand.len() >= me.hp.max(0) as usize,
        Atom::HandAtLeast(n) => me.hand.len() >= *n as usize,
        Atom::HpBelow(n) => {
            if *n == 255 {
                me.max_hp > 1
            } else {
                me.hp > 0 && me.hp < *n as i32
            }
        }
        Atom::HpEq(n) => me.hp == *n as i32,
        Atom::JudgeNonEmpty => !me.judge.is_empty(),
        Atom::EquipNonEmpty => !me.equips.is_empty(),
        Atom::EquipEmpty => me.equips.is_empty(),
        Atom::HandOrEquip => !me.hand.is_empty() || !me.equips.is_empty(),
        Atom::HandOrEquipEmpty => me.hand.is_empty() && me.equips.is_empty(),
        Atom::OtherAlive => (0..ctx.state.players.len())
            .any(|i| i as u8 != owner && ctx.state.players[i].alive),
        Atom::OtherHand => (0..ctx.state.players.len()).any(|i| {
            i as u8 != owner
                && ctx.state.players[i].alive
                && !ctx.state.players[i].hand.is_empty()
        }),
        Atom::OtherWounded => (0..ctx.state.players.len()).any(|i| {
            let p = &ctx.state.players[i];
            p.alive && p.hp > 0 && (p.hp as u32) < p.max_hp
        }),
        Atom::CardSha => sha,
        Atom::CardTrick => logical.is_some_and(kind_is_ordinary_trick),
        Atom::CardBasic => logical.is_some_and(|k| k.is_basic_kind()),
        Atom::CardEquipKind => logical.is_some_and(|k| k.equip_slot().is_some()),
        Atom::CardJuedou => logical.is_some_and(|k| matches!(k, CardKind::Duel)),
        Atom::CardRed => red,
        Atom::CardBlack => suit.is_some_and(|s| matches!(s, Suit::Spade | Suit::Club)),
        Atom::CardSpade => suit == Some(Suit::Spade),
        Atom::CardHeart => suit == Some(Suit::Heart),
        Atom::CardClub => suit == Some(Suit::Club),
        Atom::CardDiamond => suit == Some(Suit::Diamond),
        Atom::CardJiu => logical.is_some_and(|k| matches!(k, CardKind::Jiu)),
        Atom::CardShan => logical.is_some_and(|k| matches!(k, CardKind::Shan)),
        Atom::CardNanman => logical.is_some_and(|k| matches!(k, CardKind::SavageAttack)),
        Atom::CardWanjian => logical.is_some_and(|k| matches!(k, CardKind::ArcheryAttack)),
        Atom::CardWuzhong => logical.is_some_and(|k| matches!(k, CardKind::Wuzhong)),
        Atom::CardDelayed => {
            logical.is_some_and(|k| matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian))
        }
        Atom::ShaOrTrick => sha || logical.is_some_and(kind_is_ordinary_trick),
        Atom::ShaOrJuedou => sha || logical.is_some_and(|k| matches!(k, CardKind::Duel)),
        Atom::BasicOrTrick => {
            logical.is_some_and(|k| k.is_basic_kind() || kind_is_ordinary_trick(k))
        }
        Atom::ElementalOrTrick => {
            matches!(v.nature, Some(DamageNature::Fire | DamageNature::Thunder))
                || logical.is_some_and(kind_is_ordinary_trick)
        }
        Atom::RedOrJiu => red || logical.is_some_and(|k| matches!(k, CardKind::Jiu)),
        Atom::JuedouOrRedSha => {
            logical.is_some_and(|k| matches!(k, CardKind::Duel)) || (sha && red)
        }
        Atom::Fire => v.nature == Some(DamageNature::Fire),
        Atom::Thunder => v.nature == Some(DamageNature::Thunder),
        Atom::Damage => v.kind == 1 || v.nature.is_some() || v.event_amount > 0,
        Atom::PhasePlay => phase_is(ctx, v, Phase::Play),
        Atom::PhaseDraw => phase_is(ctx, v, Phase::Draw),
        Atom::PhaseDiscard => phase_is(ctx, v, Phase::Discard),
        Atom::PhaseJudge => phase_is(ctx, v, Phase::Judge),
        Atom::PhaseEnd => v.kind == 3 && v.begin == Some(false),
        Atom::PhaseBegin => v.kind == 3 && v.begin == Some(true),
        Atom::TurnDmg0 => me.turn_damage == 0,
        Atom::TurnDmg => me.turn_damage > 0,
        Atom::TurnDmgAtLeast(n) => me.turn_damage >= *n,
        Atom::BtePos => {
            ctx.state.turn.bte_used_play_phase > 0
                || ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(owner, "play_cat_mask".to_string()))
                    .copied()
                    .unwrap_or(0)
                    > 0
        }
        Atom::SourceAlive => v
            .event_source
            .is_some_and(|s| ctx.state.players.get(s as usize).is_some_and(|p| p.alive)),
        Atom::DistLe1 => {
            let other = v.event_player.filter(|p| *p != owner).or(v.event_source);
            other.is_some_and(|o| ctx.state.computed_distance(ctx.catalog, owner, o) <= 1)
        }
        Atom::TiedHand => {
            let n = me.hand.len();
            n > 0 && ctx.state.players.iter().all(|p| !p.alive || p.hand.len() <= n)
        }
        Atom::TiedHp => {
            me.hp > 0 && ctx.state.players.iter().all(|p| !p.alive || p.hp <= me.hp)
        }
        Atom::TiedEquip => {
            let n = me.equips.len();
            n > 0
                && ctx
                    .state
                    .players
                    .iter()
                    .all(|p| !p.alive || p.equips.len() <= n)
        }
        Atom::NotUniqueHp => ctx
            .state
            .players
            .iter()
            .enumerate()
            .any(|(i, p)| p.alive && i as u8 != owner && p.hp >= me.hp),
        Atom::Chain => v.from_chain,
        Atom::Direct => v.kind == 1 && !v.from_chain,
        Atom::Dying => v.kind == 6,
        Atom::Death => v.kind == 7,
        Atom::DiscardEvt => v.kind == 4 || v.batch_len > 0,
        Atom::Moved => v.kind == 5,
        Atom::Responded => v.kind == 8,
        Atom::Used => v.kind == 2,
        Atom::OutsideTurn => ctx.state.turn.current != owner,
        Atom::InTurn => ctx.state.turn.current == owner,
        Atom::Lost => ctx
            .state
            .vm_aux
            .lost
            .get(&owner)
            .is_some_and(|c| !c.is_empty()),
        Atom::LostAtLeast(n) => ctx
            .state
            .vm_aux
            .lost
            .get(&owner)
            .is_some_and(|c| c.len() >= *n as usize),
        Atom::Batch => v.batch_len > 0 || v.kind == 4 || v.kind == 5,
        Atom::PileNonEmpty => ctx
            .state
            .vm_aux
            .piles
            .iter()
            .any(|((p, _), c)| *p == owner && !c.is_empty()),
        Atom::PileEmpty => ctx
            .state
            .vm_aux
            .piles
            .iter()
            .filter(|((p, _), _)| *p == owner)
            .all(|(_, c)| c.is_empty()),
        Atom::MarkPos => ctx
            .state
            .vm_aux
            .marks
            .iter()
            .any(|((p, _), n)| *p == owner && *n > 0),
        Atom::DrawNotFixed => !me.skip_next_draw,
        Atom::Once { scope } => !ctx
            .state
            .vm_aux
            .once
            .contains(&(owner, skill.to_string(), *scope)),
        Atom::FlagSet(k) => {
            ctx.state
                .vm_aux
                .flags
                .get(&(owner, k.clone()))
                .copied()
                .unwrap_or(0)
                > 0
        }
        Atom::FlagUnset(k) => !ctx.state.vm_aux.once.contains(&(owner, k.clone(), 1)),
        Atom::Tag(k) => {
            skill.contains(k.as_str())
                || ctx.state.skills.iter().any(|((o, def), inst)| {
                    *o == owner
                        && inst.enabled
                        && ctx.catalog.skill_name(*def).contains(k.as_str())
                })
                || ctx.state.vm_aux.flags.contains_key(&(owner, k.clone()))
        }
        Atom::FirstHit => {
            ctx.state
                .vm_aux
                .hits
                .get(&(owner, skill.to_string()))
                .copied()
                .unwrap_or(0)
                == 0
        }
        Atom::SecondHit => {
            ctx.state
                .vm_aux
                .hits
                .get(&(owner, skill.to_string()))
                .copied()
                .unwrap_or(0)
                == 1
        }
        Atom::AfterFirst => {
            ctx.state
                .vm_aux
                .hits
                .get(&(owner, skill.to_string()))
                .copied()
                .unwrap_or(0)
                >= 1
        }
        Atom::OddHand => me.hand.len() % 2 == 1,
        Atom::Lord => ctx
            .state
            .identity_roles()
            .and_then(|r| r.get(owner as usize))
            .is_some_and(|r| *r == Role::Lord),
        Atom::Female => gender_of(ctx, owner) == Some(Gender::Female),
        Atom::Male => gender_of(ctx, owner) == Some(Gender::Male),
        Atom::SexSame => {
            let mine = gender_of(ctx, owner);
            v.event_player
                .or(v.event_source)
                .is_some_and(|o| o != owner && gender_of(ctx, o) == mine && mine.is_some())
        }
        Atom::GroupSame => {
            let mine = faction_of(ctx, owner);
            v.event_player
                .is_some_and(|o| faction_of(ctx, o) == mine && mine.is_some())
        }
        Atom::Transformed => v.use_as.is_some(),
        Atom::MinHp => alive && ctx.state.players.iter().all(|p| !p.alive || p.hp >= me.hp),
        Atom::TargetAlive => v
            .event_player
            .is_some_and(|p| ctx.state.players.get(p as usize).is_some_and(|pl| pl.alive)),
        Atom::TargetHandNonEmpty => v.event_player.is_some_and(|p| {
            ctx.state
                .players
                .get(p as usize)
                .is_some_and(|pl| !pl.hand.is_empty())
        }),
        Atom::TargetHandEmpty => v.event_player.is_some_and(|p| {
            ctx.state
                .players
                .get(p as usize)
                .is_some_and(|pl| pl.hand.is_empty())
        }),
        Atom::TargetEquipNonEmpty => v.event_player.is_some_and(|p| {
            ctx.state
                .players
                .get(p as usize)
                .is_some_and(|pl| !pl.equips.is_empty())
        }),
        Atom::SourceNotTarget => {
            v.event_source.is_some() && v.event_player.is_some() && v.event_source != v.event_player
        }
        Atom::CanDiscard(0) => v.event_player.is_some_and(|p| {
            ctx.state
                .players
                .get(p as usize)
                .is_some_and(|pl| !pl.hand.is_empty() || !pl.equips.is_empty())
        }),
        Atom::CanDiscard(n) => me.hand.len() + me.equips.len() >= *n as usize,
        Atom::CountAtLeast(n) => {
            v.event_amount >= *n as u32
                || me.hand.len() >= *n as usize
                || me.turn_damage >= *n
                || ctx
                    .state
                    .vm_aux
                    .lost
                    .get(&owner)
                    .is_some_and(|c| c.len() >= *n as usize)
        }
        Atom::SingleTarget => v.targets.len() <= 1,
        Atom::MultiTarget => v.targets.len() > 1,
        Atom::MovedFromOwner => v.moved_from == Some(owner),
        Atom::Gained => v.kind == 5 && v.event_player == Some(owner),
        Atom::FactionIs(tag) => faction_of(ctx, owner).is_some_and(|f| faction_tag(f) == *tag),
        Atom::SourceHasWeapon => v.event_source.is_some_and(|src| {
            ctx.state
                .players
                .get(src as usize)
                .is_some_and(|pl| {
                    pl.alive
                        && pl.equips.iter().any(|c| {
                            ctx.catalog.card_slot_of(ctx.state, *c)
                                == Some(crate::catalog::EquipSlot::Weapon)
                        })
                })
        }),
        Atom::AnyLinked => ctx.state.players.iter().any(|pl| pl.alive && pl.linked),
        Atom::DamageCard => logical.is_some_and(|k| {
            k.is_sha_like()
                || matches!(
                    k,
                    CardKind::Duel
                        | CardKind::SavageAttack
                        | CardKind::ArcheryAttack
                        | CardKind::ShanDian
                )
        }),
        Atom::NanmanWanjianOrNormalSpadeSha => {
            v.kind == 2
                && (matches!(
                    logical,
                    Some(CardKind::SavageAttack | CardKind::ArcheryAttack)
                ) || (sha
                    && v.nature == Some(DamageNature::Normal)
                    && suit == Some(Suit::Spade)))
        }
        Atom::TargetHandGteOwnerHpOrLteAttackRange => {
            let target = v
                .targets
                .first()
                .copied()
                .or_else(|| v.event_player.filter(|p| *p != owner));
            alive
                && target.is_some_and(|t| {
                    let tp = ctx.state.player(t);
                    let hand = tp.hand.len() as i32;
                    let range = ctx.state.attack_range(ctx.catalog, owner) as i32;
                    tp.alive && (hand >= me.hp || hand <= range)
                })
        }
        Atom::TargetHandLteOwnerOrHpGteOwner => {
            let target = v
                .targets
                .first()
                .copied()
                .or_else(|| v.event_player.filter(|p| *p != owner));
            alive
                && target.is_some_and(|t| {
                    let tp = ctx.state.player(t);
                    tp.alive && (tp.hand.len() <= me.hand.len() || tp.hp >= me.hp)
                })
        }
        Atom::OwnerOutsideTargetAttackRange => {
            let target = v
                .targets
                .first()
                .copied()
                .or_else(|| v.event_player.filter(|p| *p != owner));
            alive
                && target.is_some_and(|t| {
                    let tp = ctx.state.player(t);
                    tp.alive
                        && ctx.state.computed_distance(ctx.catalog, t, owner)
                            > ctx.state.attack_range(ctx.catalog, t)
                })
        }
    }
}

fn gender_of(ctx: &QueryCtx<'_>, p: PlayerId) -> Option<Gender> {
    let g = ctx.state.player(p).general_slots[0]?;
    ctx.catalog
        .general_meta(g)
        .map(|(_, _, _, _, gender)| gender)
}

fn faction_of(ctx: &QueryCtx<'_>, p: PlayerId) -> Option<Faction> {
    ctx.state.effective_faction(ctx.catalog, p)
}

fn faction_tag(f: Faction) -> u8 {
    match f {
        Faction::Wei => 0,
        Faction::Shu => 1,
        Faction::Wu => 2,
        Faction::Qun => 3,
    }
}

fn kind_is_ordinary_trick(kind: CardKind) -> bool {
    if kind.is_basic_kind() || kind.equip_slot().is_some() {
        return false;
    }
    !matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    ) && kind.is_basic_trick_or_equip()
}
