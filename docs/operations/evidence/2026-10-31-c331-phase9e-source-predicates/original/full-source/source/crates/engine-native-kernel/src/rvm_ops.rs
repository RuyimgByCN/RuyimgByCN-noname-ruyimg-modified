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
mod codec;
pub use codec::*;
mod query;
pub use query::*;

fn reject(name: &str) -> HostTrap {
    HostTrap::InvalidRequest {
        message: format!("rvm_bridge 未映射: {name}"),
    }
}

pub(crate) fn seat_of(p: rule_api::PlayerId, seats: u8) -> Option<PlayerId> {
    if !p.is_some() {
        return None;
    }
    let raw = p.raw() - 1;
    let s = u8::try_from(raw).ok()?;
    (s < seats).then_some(s)
}

fn kind_tag(kind: CardKind) -> u8 {
    CardKind::ALL.iter().position(|k| *k == kind).unwrap_or(255) as u8
}

fn skill_label(catalog: &Catalog, def: u32) -> Option<String> {
    let id = u16::try_from(def).ok()?;
    ((id as usize) < catalog.skills.len()).then(|| catalog.skill_name(id).to_string())
}

fn pile_name(catalog: &Catalog, executing: &str, pile_def: u32) -> String {
    skill_label(catalog, pile_def).unwrap_or_else(|| executing.to_string())
}

/// The private storage namespace is shared by host writes and VM queries.
pub fn storage_key(skill_name: &str, key: u32) -> String {
    format!("storage:{skill_name}#{key}")
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
        EffectRequest::DealDamage {
            target,
            amount,
            nature,
            ..
        } => {
            ok_player(*target, "DealDamage")?;
            if *amount == 0 || *amount > i32::MAX as u32 || *nature > 2 {
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
        EffectRequest::TurnOver { player } => {
            ok_player(*player, "TurnOver")?;
        }
        // PA12 批④-3：伤害改写对——只在 DamageBefore 前置窗口内合法
        //（当前帧即伤害帧；宿主技同口径直写 DamageData）。
        EffectRequest::PreventDamage { .. } => {}
        EffectRequest::ModifyCurrentDamage { player, .. } => {
            ok_player(*player, "ModifyCurrentDamage")?;
        }
        EffectRequest::SkipCurrentPhase { player } => {
            ok_player(*player, "SkipCurrentPhase")?;
        }
        // PA12 批⑤-3：DSL `storage_set(self_skill, key, value)`——技能实例
        // 私有存储（上游 player.storage.<skill>[key]）。
        EffectRequest::StorageSet { .. } => {}
        EffectRequest::ChooseGiveOrRecastOneHand { player, skill, .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player, skill, ..
        }
        | EffectRequest::ResolveOlJieDanshou { player, skill } => {
            ok_player(*player, "ResolveOlJieDanshou")?;
            if !skill.is_some() {
                return Err(reject("ResolveOlJieDanshou skill"));
            }
        }
        // PA12 批④-2：DSL `discard_hand_equip_count(player, N)`（N 为字面
        // 正数）。`count == 0` 是 VM 首发（打开选牌）——选择挂起链未接，
        // 拒绝。
        EffectRequest::DiscardHandEquipCount { player, count, .. } => {
            ok_player(*player, "DiscardHandEquipCount")?;
            if *count == 0 {
                return Err(reject("DiscardHandEquipCount 选牌首发未接"));
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
            if let Ok(def) = u16::try_from(raw) {
                if (def as usize) < ctx.catalog.skills.len() {
                    ctx.ensure_skill((p, def), SkillInstance {
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
        // PA12 批⑤-3：StorageSet——键 "{executing}#{key}" 落 vm_aux.storage
        //（VM 桥无实例 id，按执行中技能名；清零=技能自带 phase_play 触发段
        // 发 StorageSet(key,0)，旧线 DSL 同款，无需额外内核钩）。
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
        // PA12 批⑤（D 道原生化首步）：DSL `skip_current_phase(self)`——按
        // 当前阶段置既有跳过旗标（摸牌=turn.skip_draw_phase、出牌=
        // turn.skip_play_phase【摸牌阶段末消费，同乐不思蜀】、弃牌=本人
        // player.skip_discard_phase【弃牌开始即跳，同入梦】）；判定/结束
        // 阶段无旗标，静默空操作（旧线对 fenli 三窗口只用到摸/出/弃）。
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
        // PA12 批④-3：DSL `prevent_current_damage()` / `modify_current_damage(self, ±N)`
        //（旧线：damage_id=0 绑定当前前置窗口伤害帧）。与宿主技 v187/v126 同法
        // 直写 DamageData；非伤害窗口（current_damage 取不到）静默空操作。
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
                        ctx.res.fault=Some(crate::machine::VmFault{skill:"modify_current_damage".into(),owner:0,event:"Damage".into(),kind:"DamageAmountOverflow".into(),detail:"canonical i32 overflow".into()});return;
                    };
                    ctx.current_damage_mut().amount = next;
                }
            }
        }
        // PA12 批④-2：DSL `discard_hand_equip_count(player, N)`（N>0 字面）——
        // 与 DiscardFromHandEquip 同口径（手牌优先确定性首 N、装备补足、
        // 数量不足尽弃）；N=0 选牌首发在 accept 阶段拒绝。
        EffectRequest::DiscardHandEquipCount { player, count, .. } => {
            let count = count.max(1); // N=0 已在 accept 拒绝，防御取正
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
        // PA12 批④-1：DSL `turn_over(player)`——经内核翻面提交落点
        // （神裔守卫、急冻解除、TurnOvered 事件同口径）。
        EffectRequest::TurnOver { player } => {
            if let Some(p) = seat_of(player, seats) {
                // 与 DealDamage 同法：发射 TurnOverBefore 子事件帧，由内核
                // Commit 段执行翻面（神裔守卫/急冻解除/TurnOvered 同口径）。
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
