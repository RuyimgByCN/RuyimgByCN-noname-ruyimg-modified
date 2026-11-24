//! Batch Q4 `jiwu` 派生技真实实现（`qiangxix` / `retieji` / `olxuanfeng` / `rexuanfeng` / `rewansha` / `boss_baonuwash`）。

use super::super::super::*;
use super::q4_boss_extra::{discard_owned_card_and_emit, is_equip_slot_open};
use super::{
    wait_q4_boss, ASK_BAONU_FORM, ASK_QIANGXIX_CARD, ASK_RETIEJI_CONFIRM, ASK_RETIEJI_DISCARD,
    ASK_RETIEJI_GIVE, ASK_RETIEJI_JUDGE, ASK_REXUANFENG_MOVE_CARD, ASK_REXUANFENG_MOVE_FROM,
    ASK_REXUANFENG_MOVE_TO, ASK_XUANFENG_TARGET,
};
use crate::catalog::{Catalog, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, LostOrigin};
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(super) fn suit_to_code(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn code_to_suit(code: u32) -> Option<Suit> {
    match code {
        1 => Some(Suit::Spade),
        2 => Some(Suit::Heart),
        3 => Some(Suit::Club),
        4 => Some(Suit::Diamond),
        _ => None,
    }
}

pub(super) fn has_other_card_target(core: &GameStateCore, owner: PlayerId) -> bool {
    (0..core.players.len() as PlayerId).any(|p| {
        p != owner
            && core.player(p).alive
            && (!core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
    })
}

pub(super) fn qiangxix_targets(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    (0..core.players.len() as PlayerId)
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && core
                    .vm_aux
                    .flags
                    .get(&(owner, format!("skill_turn:q4_qiangxix_hit:{p}")))
                    .copied()
                    .unwrap_or(0)
                    == 0
        })
        .collect()
}

pub(super) fn qiangxix_weapons(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .chain(core.player(owner).equips.iter())
        .copied()
        .filter(|&c| catalog.card_slot_of(core, c) == Some(crate::catalog::EquipSlot::Weapon))
        .collect()
}

pub(super) fn can_activate_qiangxix(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    let uses = core
        .vm_aux
        .flags
        .get(&(owner, "skill_turn:q4_qiangxix_uses".to_string()))
        .copied()
        .unwrap_or(0);
    if uses >= 2 {
        return false;
    }
    if core.player(owner).hp < 1 && qiangxix_weapons(core, catalog, owner).is_empty() {
        return false;
    }
    !qiangxix_targets(core, owner).is_empty()
}

fn commit_qiangxix_mark_used(ctx: &mut Ctx<'_>, owner: PlayerId, tgt: PlayerId) {
    *ctx.core
        .vm_aux
        .flags
        .entry((owner, "skill_turn:q4_qiangxix_uses".to_string()))
        .or_insert(0) += 1;
    ctx.core
        .vm_aux
        .flags
        .insert((owner, format!("skill_turn:q4_qiangxix_hit:{tgt}")), 1);
    ctx.log(LogEvent::SkillTriggered {
        skill: "qiangxix",
        owner,
        presentation: None,
    });
}

pub(super) fn step_qiangxix_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !qiangxix_targets(ctx.core, owner).contains(&tgt) {
        return Flow::Done;
    }
    m.target = Some(tgt);
    let weapons = qiangxix_weapons(ctx.core, ctx.catalog, owner);
    let hp = ctx.core.player(owner).hp;
    if hp >= 1 && weapons.is_empty() {
        commit_qiangxix_mark_used(ctx, owner, tgt);
        m.card = Some(ASK_QIANGXIX_CARD);
        m.stage = HostStage::DrawSettle;
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: 1,
        });
        return Flow::Yield;
    }
    if hp < 1 && weapons.is_empty() {
        return Flow::Done;
    }
    m.card = Some(ASK_QIANGXIX_CARD);
    m.stage = HostStage::SelectCard;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_qiangxix_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(tgt) = m.target else {
        return Flow::Done;
    };
    let hp = ctx.core.player(owner).hp;
    let picked = ctx.answer().iter().find_map(|o| match *o {
        ChoiceOption::Card(c)
            if ctx.core.catalog_contains_weapon_owned(ctx.catalog, owner, c) =>
        {
            Some(c)
        }
        _ => None,
    });
    if let Some(weapon) = picked {
        commit_qiangxix_mark_used(ctx, owner, tgt);
        discard_owned_card_and_emit(ctx, owner, weapon);
        m.card = Some(ASK_QIANGXIX_CARD);
        m.stage = HostStage::DrawSettle;
        return Flow::Yield;
    }
    if hp < 1 {
        return Flow::Done;
    }
    commit_qiangxix_mark_used(ctx, owner, tgt);
    m.card = Some(ASK_QIANGXIX_CARD);
    m.stage = HostStage::DrawSettle;
    ctx.emit(EventData::LoseHp {
        player: owner,
        amount: 1,
    });
    Flow::Yield
}

pub(super) fn finish_qiangxix_damage(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(tgt) = m.target {
        if ctx.core.player(tgt).alive {
            ctx.emit(EventData::Damage(damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Normal,
                None,
            )));
        }
    }
    Flow::Done
}

trait WeaponOwnedExt {
    fn catalog_contains_weapon_owned(
        &self,
        catalog: &Catalog,
        owner: PlayerId,
        c: CardId,
    ) -> bool;
}

impl WeaponOwnedExt for GameStateCore {
    fn catalog_contains_weapon_owned(&self, catalog: &Catalog, owner: PlayerId, c: CardId) -> bool {
        let pl = self.player(owner);
        (pl.hand.contains(&c) || pl.equips.contains(&c))
            && catalog.card_slot_of(self, c) == Some(crate::catalog::EquipSlot::Weapon)
    }
}

pub(crate) fn retieji_sha_begin(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
) -> bool {
    let Some(def) = ctx.catalog.skill_by_name("retieji") else {
        return false;
    };
    if ctx.catalog.skills[def as usize].behavior.as_host_behavior()
        != Some(HostBehavior::Q4BossPlaySkill)
    {
        return false;
    }
    if !ctx.core.has_skill(source, def)
        || ctx.catalog.is_tieji_sealed(ctx.core, source)
        || ctx
            .catalog
            .is_rechanyuan_sealed_name(ctx.core, source, "retieji")
    {
        return false;
    }
    let done_key = (source, format!("q4_retieji_done:{sha_frame_id}:{target}"));
    if ctx.core.vm_aux.flags.contains_key(&done_key) {
        return false;
    }
    ctx.core.vm_aux.flags.insert(done_key, 1);
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger {
            frame: sha_frame_id,
        },
        machine: Box::new(HostMachine {
            def_key: "retieji".to_string(),
            behavior: HostBehavior::Q4BossPlaySkill,
            owner: source,
            target: Some(target),
            card: Some(ASK_RETIEJI_CONFIRM),
            cards: vec![sha_card.unwrap_or(0)],
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
        awaiting_answer: false,
    });
    true
}

pub(super) fn retieji_matching_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    suit: Suit,
) -> Vec<CardId> {
    let pl = core.player(target);
    pl.hand
        .iter()
        .chain(pl.equips.iter())
        .copied()
        .filter(|&c| catalog.effective_card_suit(core, target, c) == suit)
        .collect()
}

fn is_l03_retieji(catalog: &Catalog) -> bool {
    let Some(rtj) = catalog.skill_by_name("retieji") else {
        return false;
    };
    let Some(rexf) = catalog.skill_by_name("rexuanfeng") else {
        return false;
    };
    let has_u02 = catalog.skill_by_name("olxuanfeng").is_some()
        || catalog.skill_by_name("rewansha").is_some();
    !has_u02 || rexf == rtj + 1
}

fn mark_retieji_fail(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    tgt: PlayerId,
    sha_cid: CardId,
    add_damage: bool,
) {
    ctx.core
        .vm_aux
        .flags
        .insert((owner, format!("sha_unrespondable:{sha_cid}:{tgt}")), 1);
    if add_damage {
        *ctx.core
            .vm_aux
            .flags
            .entry((owner, format!("sha_target_damage_bonus:{sha_cid}:{tgt}")))
            .or_insert(0) += 1;
    }
}

pub(super) fn step_retieji_confirm(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(tgt) = m.target else {
        return Flow::Done;
    };
    if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm))
        || !ctx.core.player(owner).alive
        || !ctx.core.player(tgt).alive
    {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "retieji",
        owner,
        presentation: None,
    });
    m.card = Some(ASK_RETIEJI_JUDGE);
    m.stage = HostStage::DrawSettle;
    ctx.emit(EventData::OrdinaryJudge {
        owner,
        invocation: u32::MAX,
    });
    Flow::Yield
}

pub(super) fn step_retieji_judge_settled(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let suit_code = ctx
        .core
        .vm_aux
        .flags
        .remove(&(owner, "last_judge_suit".to_string()))
        .unwrap_or(0);
    let _ = ctx
        .core
        .vm_aux
        .flags
        .remove(&(owner, "last_judge_rank".to_string()));
    let judged_card = ctx
        .core
        .vm_aux
        .flags
        .remove(&(owner, "last_judge_card".to_string()));
    let Some(tgt) = m.target else {
        return Flow::Done;
    };
    let Some(suit) = code_to_suit(suit_code).or_else(|| {
        judged_card.map(|jc| ctx.catalog.effective_card_suit(ctx.core, owner, jc))
    }) else {
        return Flow::Done;
    };
    if !ctx.core.player(owner).alive || !ctx.core.player(tgt).alive {
        return Flow::Done;
    }
    ctx.core
        .vm_aux
        .flags
        .insert((tgt, "q4_retieji_target_sealed".to_string()), 1);
    let sha_cid = m.cards.first().copied().unwrap_or(0);
    let l03 = is_l03_retieji(ctx.catalog);
    if retieji_matching_cards(ctx.core, ctx.catalog, tgt, suit).is_empty() {
        mark_retieji_fail(ctx, owner, tgt, sha_cid, !l03);
        return Flow::Done;
    }
    m.cards = vec![sha_cid, suit_to_code(suit)];
    m.card = Some(if l03 {
        ASK_RETIEJI_DISCARD
    } else {
        ASK_RETIEJI_GIVE
    });
    m.stage = HostStage::SelectCard;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_retieji_give(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    resolve_retieji_cost(m, ctx, false)
}

pub(super) fn step_retieji_discard(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    resolve_retieji_cost(m, ctx, true)
}

fn resolve_retieji_cost(m: &HostMachine, ctx: &mut Ctx<'_>, discard: bool) -> Flow {
    let owner = m.owner;
    let Some(tgt) = m.target else {
        return Flow::Done;
    };
    let sha_cid = m.cards.first().copied().unwrap_or(0);
    let Some(suit) = m.cards.get(1).copied().and_then(code_to_suit) else {
        mark_retieji_fail(ctx, owner, tgt, sha_cid, !discard);
        return Flow::Done;
    };
    let valid = retieji_matching_cards(ctx.core, ctx.catalog, tgt, suit);
    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
        if valid.contains(&c) {
            if discard {
                discard_owned_card_and_emit(ctx, tgt, c);
                (ctx.catalog.hooks.on_cards_to_discard)(ctx, tgt, &[c], false);
                return Flow::Done;
            }
            if ctx.core.player(tgt).equips.contains(&c) {
                ctx.core.player_mut(tgt).equips.retain(|x| *x != c);
                ctx.core.declared_slots.remove(&c);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, tgt, c);
                ctx.note_cards_lost(tgt, &[c]);
                ctx.emit(EventData::CardsLost {
                    player: tgt,
                    cards: vec![c],
                    origins: vec![LostOrigin::Equip],
                });
                ctx.core.player_mut(owner).hand.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: tgt,
                    to: owner,
                });
                ctx.emit(EventData::CardsGiven {
                    giver: tgt,
                    recipient: owner,
                    cards: vec![c],
                });
                ctx.log(LogEvent::CardsGiven {
                    from: tgt,
                    to: owner,
                    cards: vec![c],
                });
            } else {
                ctx.give_hand_card(tgt, owner, c);
            }
            return Flow::Done;
        }
    }
    mark_retieji_fail(ctx, owner, tgt, sha_cid, !discard);
    Flow::Done
}

pub(super) fn can_move_other_equip(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    !rexuanfeng_move_from_candidates(core, catalog, owner).is_empty()
}

pub(super) fn rexuanfeng_move_from_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    (0..core.players.len() as PlayerId)
        .filter(|&from| {
            from != owner
                && core.player(from).alive
                && !rexuanfeng_movable_cards(core, catalog, owner, from).is_empty()
        })
        .collect()
}

pub(super) fn rexuanfeng_movable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    from: PlayerId,
) -> Vec<CardId> {
    core.player(from)
        .equips
        .iter()
        .copied()
        .filter(|&eq| !rexuanfeng_move_to_candidates(core, catalog, owner, from, eq).is_empty())
        .collect()
}

pub(super) fn rexuanfeng_move_to_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    from: PlayerId,
    eq_card: CardId,
) -> Vec<PlayerId> {
    let Some(slot) = catalog.card_slot_of(core, eq_card) else {
        return Vec::new();
    };
    (0..core.players.len() as PlayerId)
        .filter(|&to| {
            to != owner
                && to != from
                && core.player(to).alive
                && is_equip_slot_open(core, catalog, to, slot)
        })
        .collect()
}

pub(super) fn step_rexuanfeng_mode(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(1))
            if can_move_other_equip(ctx.core, ctx.catalog, owner) =>
        {
            m.card = Some(ASK_REXUANFENG_MOVE_FROM);
            m.stage = HostStage::TargetChoice;
            wait_q4_boss(m, ctx)
        }
        Some(ChoiceOption::Tag(0)) if has_other_card_target(ctx.core, owner) => {
            m.cards = vec![2];
            m.card = Some(ASK_XUANFENG_TARGET);
            m.stage = HostStage::TargetChoice;
            wait_q4_boss(m, ctx)
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_rexuanfeng_move_from(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(ChoiceOption::Player(from)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !rexuanfeng_move_from_candidates(ctx.core, ctx.catalog, owner).contains(&from) {
        return Flow::Done;
    }
    m.target = Some(from);
    m.card = Some(ASK_REXUANFENG_MOVE_CARD);
    m.stage = HostStage::SelectCard;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_rexuanfeng_move_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(from) = m.target else {
        return Flow::Done;
    };
    let Some(ChoiceOption::Card(eq)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !rexuanfeng_movable_cards(ctx.core, ctx.catalog, owner, from).contains(&eq) {
        return Flow::Done;
    }
    m.cards = vec![eq];
    m.card = Some(ASK_REXUANFENG_MOVE_TO);
    m.stage = HostStage::CompareTargetPick;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_rexuanfeng_move_to(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let (Some(from), Some(eq), Some(ChoiceOption::Player(to))) = (
        m.target,
        m.cards.first().copied(),
        ctx.answer().first().copied(),
    ) else {
        return Flow::Done;
    };
    if !rexuanfeng_move_to_candidates(ctx.core, ctx.catalog, owner, from, eq).contains(&to) {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "rexuanfeng",
        owner,
        presentation: None,
    });
    let preserved_slot = ctx.core.declared_slots.get(&eq).copied();
    ctx.core.player_mut(from).equips.retain(|x| *x != eq);
    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, eq);
    ctx.note_cards_lost(from, &[eq]);
    if let Some(slot) = preserved_slot {
        ctx.core.declared_slots.insert(eq, slot);
    }
    ctx.core.player_mut(to).equips.push(eq);
    crate::kernel::grant_equip_skills(ctx, to, eq);
    ctx.emit(EventData::CardsLost {
        player: from,
        cards: vec![eq],
        origins: vec![LostOrigin::Equip],
    });
    ctx.emit(EventData::CardsMoved {
        cards: vec![eq],
        from,
        to,
    });
    Flow::Done
}

pub(super) fn start_rewansha(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if ctx.core.turn.current != owner || !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "rewansha",
        owner,
        presentation: None,
    });
    for p in 0..ctx.core.players.len() as PlayerId {
        if p != owner && Some(p) != m.target && ctx.core.player(p).alive {
            ctx.core
                .vm_aux
                .flags
                .insert((p, "q4_rewansha_sealed".to_string()), 1);
        }
    }
    Flow::Done
}

/// Hook for `boss_baonuwash` (`apps/core/mode/boss.js:9730-9768`):
/// 1st deck wash while Boss holds `boss_baonu` immediately triggers `boss_baonu`;
/// 2nd+ deck wash grants `boss_baonuwash` to the active turn player so Boss wins at `phaseAfter`.
pub(crate) fn q4_boss_on_deck_washed(ctx: &mut Ctx<'_>) {
    let Some((boss, _)) = ctx.core.boss_mode() else {
        return;
    };
    if !ctx.core.player(boss).alive {
        return;
    }
    let has_baonu = ctx
        .catalog
        .skill_by_name("boss_baonu")
        .is_some_and(|def| ctx.core.has_skill(boss, def));
    let armed = ctx
        .core
        .vm_aux
        .flags
        .get(&(0, "q4_boss_baonuwash_armed".to_string()))
        .copied()
        .unwrap_or(0)
        != 0;
    if !armed && !has_baonu {
        return;
    }
    ctx.core
        .vm_aux
        .flags
        .insert((0, "q4_boss_baonuwash_armed".to_string()), 1);
    let already_triggered = ctx
        .core
        .vm_aux
        .flags
        .get(&(0, "q4_boss_baonuwash_triggered".to_string()))
        .copied()
        .unwrap_or(0)
        != 0;
    if !already_triggered {
        ctx.core
            .vm_aux
            .flags
            .insert((0, "q4_boss_baonuwash_triggered".to_string()), 1);
        if !has_baonu {
            return;
        }
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Intrinsic {
                frame: crate::resolution::NO_FRAME,
            },
            machine: Box::new(HostMachine {
                def_key: "boss_baonu".to_string(),
                behavior: HostBehavior::Q4BossPlaySkill,
                owner: boss,
                target: None,
                card: Some(ASK_BAONU_FORM),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }),
            awaiting_answer: false,
        });
    } else {
        let cur = ctx.core.turn.current;
        if (cur as usize) < ctx.core.players.len() {
            ctx.core
                .vm_aux
                .flags
                .insert((cur, "q4_boss_baonuwash_game_over".to_string()), 1);
        }
    }
}
