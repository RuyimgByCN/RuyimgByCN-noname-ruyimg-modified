//! Package active entry constraints and trigger handlers sharing one logical skill.
use super::modifiers::VmModifierRule;
use super::*;
use crate::catalog::{Catalog, EquipSlot, Suit, TriggerMeta};
use crate::state::GameStateCore;

mod active_specs;
pub use active_specs::*;


fn fnv1a_32_str(s: &str) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    for &b in s.as_bytes() {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn owner_has_skill_hash(state: &GameStateCore, catalog: &Catalog, owner: PlayerId, target_hash: u32) -> bool {
    if state
        .vm_aux
        .flags
        .get(&(owner, format!("skill:{target_hash}")))
        .copied()
        .unwrap_or(0)
        != 0
    {
        return true;
    }
    state.skills.iter().any(|((p, def), inst)| {
        *p == owner
            && inst.enabled
            && (*def as u32 == target_hash
                || catalog
                    .skills
                    .get(*def as usize)
                    .is_some_and(|d| {
                        d.behavior.package_skill_id() == Some(target_hash)
                            || fnv1a_32_str(d.name) == target_hash
                    }))
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VmActiveLimits {
    pub once_per_turn: bool,
    pub once_per_play_phase: bool,
    pub once_per_game: bool,
    pub uses_per_turn: Option<u8>,
    pub uses_per_play_phase: Option<u8>,
    pub uses_per_turn_source: Option<VmTurnUseSource>,
    pub hand_nonempty: bool,
    pub equip_nonempty: bool,
    pub hand_gt_hp: bool,
    pub hand_gt_max_hp: bool,
    pub face_up: bool,
    pub owner_unlinked: bool,
    pub mark_at_least: Option<(u32, u32)>,
    pub require_missing_skill: u32,
    pub require_hand_limit_positive: bool,
    pub require_hand_missing_gaintag: u32,
    pub target_other: bool,
    pub target_max_hp_gt: Option<u8>,
    pub selection: Option<VmActiveSelection>,
    pub targets: Option<VmActiveTargets>,
}

impl VmActiveLimits {
    pub fn record_active_halt(
        &self,
        core: &mut GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_name: &str,
        selected_cards: &[u32],
        selected_targets: &[u32],
        seats: u8,
    ) {
        if let Some(sel) = self.selection {
            if sel.unique_category_in_turn {
                let key = (owner, format!("unique_cat_mask:{skill_name}"));
                let mut mask = core.vm_aux.flags.get(&key).copied().unwrap_or(0);
                for &cid in selected_cards {
                    mask |= VmActiveSelection::card_category_bit(catalog, cid);
                }
                core.vm_aux.flags.insert(key, mask);
            }
            if sel.unused_category_by_skill_this_play_phase {
                for &cid in selected_cards {
                    let bit = VmActiveSelection::card_category_bit(catalog, cid);
                    if bit != 0 {
                        core.vm_aux
                            .once
                            .insert((owner, format!("unique_cat_play:{skill_name}:{bit}"), 4));
                    }
                }
            }
        }
        if let Some(tgts) = self.targets {
            for &raw in selected_targets {
                if let Some(q) = crate::rvm_ops::seat_of(rule_api::PlayerId::new(raw), seats) {
                    if tgts.filter.unique_in_turn {
                        core.vm_aux
                            .once
                            .insert((owner, format!("active_target:{skill_name}:{q}"), 1));
                    }
                    if tgts.filter.unique_in_play_phase {
                        core.vm_aux
                            .once
                            .insert((owner, format!("active_target:{skill_name}:{q}"), 4));
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsWindow {
    PlayPhase,
    Response,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsZone {
    Hand,
    Equip,
    HandEquip,
    HandExpansion,
    HandEquipExpansion,
    GroupWei,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsCost {
    GainMaxHp1,
    DisableEquipSlot,
    Guhuo,
    FaceDownHand,
    SwapLowerHand,
    DrawOneOutsideTurnEmptyHand,
    HandEquipMark,
    LoseMaxHp1,
    AllHand,
    Potdaozhuan,
    Dczhaowen,
    Dddshilie,
    Dddzhijie,
    Dddfusi,
    Dddtongyu,
    TurnOver,
    Potqingyan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsMultiCardBonus {
    Damage1,
    Recover1,
    DiscardCurrentHandEquip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsFilter {
    Any,
    Red,
    Black,
    Spade,
    Heart,
    Club,
    Diamond,
    OwnerDyingClub,
    OwnerOutsideTurn,
    OwnerOutsideTurnRed,
    OwnerSingleHand,
    OwnerSingleRedHand,
    OwnerHandGtHp,
    OwnerHpLeTwoBlackTrick,
    Basic,
    NonBasic,
    Equip,
    NonEquip,
    Trick,
    DamageTrick,
    Sha,
    Shan,
    Tao,
    Jiu,
    CardYing,
    ColorNeShuangxiong,
    Weapon,
    Armor,
    Mount,
    BlackBasicEquip,
    RedBasic,
    BlackBasic,
    RedTrick,
    BlackTrick,
    RedNonBasic,
    BlackNonBasic,
}

impl VmViewAsFilter {
    pub fn matches(
        self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        card: CardId,
    ) -> bool {
        let Some(def) = catalog.cards.get(card as usize) else {
            return false;
        };
        let kind = def.kind;
        let suit = def.suit;
        let is_red = matches!(suit, Suit::Heart | Suit::Diamond);
        let is_black = matches!(suit, Suit::Spade | Suit::Club);
        let is_trick = !kind.is_basic_kind() && !kind.is_equip();
        let p = state.player(owner);
        match self {
            Self::Any => true,
            Self::Red => is_red,
            Self::Black => is_black,
            Self::Spade => suit == Suit::Spade,
            Self::Heart => suit == Suit::Heart,
            Self::Club => suit == Suit::Club,
            Self::Diamond => suit == Suit::Diamond,
            Self::OwnerDyingClub => p.hp <= 0 && suit == Suit::Club,
            Self::OwnerOutsideTurn => state.turn.current != owner,
            Self::OwnerOutsideTurnRed => state.turn.current != owner && is_red,
            Self::OwnerSingleHand => p.hand.len() == 1,
            Self::OwnerSingleRedHand => {
                is_red
                    && p.hand
                        .iter()
                        .filter(|&&c| {
                            catalog
                                .cards
                                .get(c as usize)
                                .is_some_and(|d| matches!(d.suit, Suit::Heart | Suit::Diamond))
                        })
                        .count()
                        == 1
            }
            Self::OwnerHandGtHp => (p.hand.len() as i32) > p.hp,
            Self::OwnerHpLeTwoBlackTrick => p.hp <= 2 && is_black && is_trick,
            Self::Basic => kind.is_basic_kind(),
            Self::NonBasic => !kind.is_basic_kind(),
            Self::Equip => kind.is_equip(),
            Self::NonEquip => !kind.is_equip(),
            Self::Trick => is_trick,
            Self::DamageTrick => matches!(
                kind,
                crate::catalog::CardKind::SavageAttack
                    | crate::catalog::CardKind::ArcheryAttack
                    | crate::catalog::CardKind::Duel
                    | crate::catalog::CardKind::ShanDian
                    | crate::catalog::CardKind::HongShui
                    | crate::catalog::CardKind::Linghunzhihuo
                    | crate::catalog::CardKind::Shenhuofeiya
                    | crate::catalog::CardKind::Shatang
                    | crate::catalog::CardKind::Yuansuhuimie
                    | crate::catalog::CardKind::Aozuzhilei
                    | crate::catalog::CardKind::Liuxinghuoyu
                    | crate::catalog::CardKind::Jingleishan
                    | crate::catalog::CardKind::Chiyuxi
            ),
            Self::Sha => kind.is_sha_like(),
            Self::Shan => kind == crate::catalog::CardKind::Shan,
            Self::Tao => kind == crate::catalog::CardKind::Tao,
            Self::Jiu => kind == crate::catalog::CardKind::Jiu,
            Self::CardYing => def.name == "ying",
            Self::ColorNeShuangxiong => {
                let recorded = state
                    .vm_aux
                    .marks
                    .get(&(owner, 600_000_511))
                    .copied()
                    .or_else(|| {
                        state
                            .vm_aux
                            .flags
                            .get(&(owner, "shuangxiong_color".to_string()))
                            .copied()
                            .map(|v| v as i32)
                    })
                    .unwrap_or(0);
                match recorded {
                    1 => is_black,
                    2 => is_red,
                    _ => false,
                }
            }
            Self::Weapon => kind.equip_slot() == Some(EquipSlot::Weapon),
            Self::Armor => kind.equip_slot() == Some(EquipSlot::Armor),
            Self::Mount => matches!(
                kind.equip_slot(),
                Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
            ),
            Self::BlackBasicEquip => is_black && (kind.is_basic_kind() || kind.is_equip()),
            Self::RedBasic => is_red && kind.is_basic_kind(),
            Self::BlackBasic => is_black && kind.is_basic_kind(),
            Self::RedTrick => is_red && is_trick,
            Self::BlackTrick => is_black && is_trick,
            Self::RedNonBasic => is_red && !kind.is_basic_kind(),
            Self::BlackNonBasic => is_black && !kind.is_basic_kind(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmViewAsSpec {
    pub skill_def_id: u32,
    pub skill_name: String,
    pub as_kind: crate::catalog::CardKind,
    pub window: VmViewAsWindow,
    pub zone: VmViewAsZone,
    pub filter: VmViewAsFilter,
    pub uses_per_turn: Option<u8>,
    pub ignore_distance: bool,
    pub red_unrespondable: bool,
    pub black_no_use_count: bool,
    pub no_use_count: bool,
    pub target_lacks_source_suit_unrespondable: bool,
    pub refund_use_count_if_no_damage: bool,
    pub min_cards: u8,
    pub max_cards: u8,
    pub owner_hp_min_one_cards: bool,
    pub owner_basic_uses_plus_one_cards: bool,
    pub once_per_turn_per_branch: bool,
    pub same_suit: bool,
    pub multi_card_bonus: Option<VmViewAsMultiCardBonus>,
    pub branch: Option<u8>,
    pub once_per_round: bool,
    pub next_branch: Option<u8>,
    pub cost: Option<VmViewAsCost>,
    pub mark_cost: Option<u32>,
    pub post_draw: u8,
}

impl VmViewAsSpec {
    pub fn required_cards(&self, state: &GameStateCore, owner: PlayerId) -> usize {
        if self.zone == VmViewAsZone::None {
            return 0;
        }
        if self.owner_hp_min_one_cards {
            return state.player(owner).hp.max(1) as usize;
        }
        if self.owner_basic_uses_plus_one_cards {
            return state.player(owner).total_used.saturating_add(1) as usize;
        }
        (self.min_cards as usize).max(1)
    }

    pub fn zone_cards(&self, state: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
        let p = state.player(owner);
        let expansion = || {
            state
                .vm_aux
                .piles
                .iter()
                .filter(|((po, _), _)| *po == owner)
                .flat_map(|(_, pile)| pile.iter().copied())
                .collect::<Vec<_>>()
        };
        match self.zone {
            VmViewAsZone::Hand => p.hand.clone(),
            VmViewAsZone::Equip => p.equips.clone(),
            VmViewAsZone::HandEquip => {
                let mut v = p.hand.clone();
                v.extend(p.equips.iter().copied());
                v
            }
            VmViewAsZone::HandExpansion => {
                let mut v = p.hand.clone();
                v.extend(expansion());
                v
            }
            VmViewAsZone::HandEquipExpansion => {
                let mut v = p.hand.clone();
                v.extend(p.equips.iter().copied());
                v.extend(expansion());
                v
            }
            VmViewAsZone::GroupWei => {
                let mut v = p.hand.clone();
                for (idx, other) in state.players.iter().enumerate() {
                    if idx as PlayerId != owner && other.alive {
                        v.extend(other.hand.iter().copied());
                    }
                }
                v
            }
            VmViewAsZone::None => Vec::new(),
        }
    }

    pub fn matches_card(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        card: CardId,
        in_play: bool,
        target_kind: Option<crate::catalog::CardKind>,
    ) -> bool {
        let p = state.player(owner);
        if !p.alive {
            return false;
        }
        if state
            .vm_aux
            .flags
            .get(&(owner, format!("skill_banned_turn:{}", self.skill_name)))
            .copied()
            .unwrap_or(0)
            != 0
        {
            return false;
        }
        let window_ok = if in_play {
            matches!(self.window, VmViewAsWindow::PlayPhase | VmViewAsWindow::Any)
        } else {
            matches!(self.window, VmViewAsWindow::Response | VmViewAsWindow::Any)
        };
        if !window_ok {
            return false;
        }
        if let Some(tk) = target_kind {
            let kind_ok = if tk == crate::catalog::CardKind::Sha {
                self.as_kind.is_sha_like()
            } else {
                self.as_kind == tk
            };
            if !kind_ok {
                return false;
            }
        } else if in_play {
            if matches!(
                self.as_kind,
                crate::catalog::CardKind::Shan | crate::catalog::CardKind::Nullify
            ) {
                return false;
            }
            if self.as_kind == crate::catalog::CardKind::Tao && p.hp >= p.max_hp as i32 {
                return false;
            }
            if self.as_kind == crate::catalog::CardKind::Jiu && state.turn.jiu_used {
                return false;
            }
        }
        if let Some(max) = self.uses_per_turn {
            let key = format!("view_as:{}", self.skill_name);
            if state.vm_aux.once_count(owner, 1, &key) >= max as usize {
                return false;
            }
        }
        if self.once_per_round {
            let key = if let Some(b) = self.branch {
                format!("view_as_round:{}:{b}", self.skill_name)
            } else {
                format!("view_as_round:{}", self.skill_name)
            };
            if state.vm_aux.once_count(owner, 2, &key) > 0 {
                return false;
            }
        }
        if self.once_per_turn_per_branch {
            let b = self.branch.unwrap_or(self.as_kind as u8);
            let key = format!("view_as_branch_turn:{}:{b}", self.skill_name);
            if state.vm_aux.once_count(owner, 1, &key) > 0 {
                return false;
            }
        }
        if let Some(b) = self.branch {
            let cur = state
                .vm_aux
                .flags
                .get(&(owner, format!("view_as_branch:{}", self.skill_name)))
                .copied()
                .unwrap_or(0) as u8;
            if cur != b {
                return false;
            }
        }
        if let Some(mark) = self.mark_cost {
            if state.vm_aux.marks.get(&(owner, mark)).copied().unwrap_or(0) == 0 {
                return false;
            }
        }
        if let Some(cost) = self.cost {
            let cost_ok = match cost {
                VmViewAsCost::TurnOver => !p.face_down,
                VmViewAsCost::LoseMaxHp1 => p.max_hp > 1,
                VmViewAsCost::DisableEquipSlot => {
                    let mask = state
                        .vm_aux
                        .flags
                        .get(&(owner, "disabled_equip_slots".to_string()))
                        .copied()
                        .unwrap_or(0);
                    (mask & 0b11_1110) != 0b11_1110
                }
                VmViewAsCost::AllHand
                | VmViewAsCost::FaceDownHand
                | VmViewAsCost::Guhuo
                | VmViewAsCost::Dczhaowen
                | VmViewAsCost::SwapLowerHand => !p.hand.is_empty(),
                VmViewAsCost::HandEquipMark => !p.hand.is_empty() || !p.equips.is_empty(),
                VmViewAsCost::DrawOneOutsideTurnEmptyHand => {
                    state.turn.current != owner && p.hand.is_empty()
                }
                _ => true,
            };
            if !cost_ok {
                return false;
            }
        }
        if self.zone == VmViewAsZone::None {
            return !p.hand.contains(&card)
                && !p.equips.contains(&card)
                && (card as usize) < catalog.cards.len();
        }
        let zcards = self.zone_cards(state, owner);
        if !zcards.contains(&card) || !self.filter.matches(state, catalog, owner, card) {
            return false;
        }
        let req = self.required_cards(state, owner);
        if req > 1 {
            let primary_suit = catalog.cards.get(card as usize).map(|d| d.suit);
            let companions = zcards
                .iter()
                .filter(|&&c2| {
                    c2 != card
                        && self.filter.matches(state, catalog, owner, c2)
                        && (!self.same_suit
                            || catalog.cards.get(c2 as usize).map(|d| d.suit) == primary_suit)
                })
                .count();
            if companions < req - 1 {
                return false;
            }
        }
        true
    }

    pub fn consume_and_record(
        &self,
        core: &mut GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        primary_card: CardId,
    ) -> Vec<CardId> {
        let mut extra = Vec::new();
        if self.zone != VmViewAsZone::None {
            let req = self.required_cards(core, owner);
            let want = if self.multi_card_bonus.is_some() && self.max_cards >= 2 {
                req.max(2)
            } else {
                req
            };
            if want > 1 {
                let primary_suit = catalog.cards.get(primary_card as usize).map(|d| d.suit);
                let zcards = self.zone_cards(core, owner);
                for c2 in zcards {
                    if extra.len() >= want - 1 {
                        break;
                    }
                    if c2 != primary_card
                        && !extra.contains(&c2)
                        && self.filter.matches(core, catalog, owner, c2)
                        && (!self.same_suit
                            || catalog.cards.get(c2 as usize).map(|d| d.suit) == primary_suit)
                    {
                        extra.push(c2);
                    }
                }
                for &c2 in &extra {
                    core.player_mut(owner).hand.retain(|&x| x != c2);
                    if core.player(owner).equips.contains(&c2) {
                        core.player_mut(owner).equips.retain(|&x| x != c2);
                        core.declared_slots.remove(&c2);
                        crate::kernel::revoke_equip_skills(core, catalog, owner, c2);
                    }
                    for ((po, _), pile) in core.vm_aux.piles.iter_mut() {
                        if *po == owner {
                            pile.retain(|&x| x != c2);
                        }
                    }
                }
            }
            if !extra.is_empty() {
                if let Some(bonus) = self.multi_card_bonus {
                    match bonus {
                        VmViewAsMultiCardBonus::Damage1 => {
                            core.turn.jiu_active = true;
                        }
                        VmViewAsMultiCardBonus::Recover1 => {
                            core.vm_aux
                                .flags
                                .insert((owner, "vm_va_recover_bonus".to_string()), 1);
                        }
                        VmViewAsMultiCardBonus::DiscardCurrentHandEquip => {
                            let cur = core.turn.current;
                            if (cur as usize) < core.players.len() && core.player(cur).alive {
                                if let Some(c) = core.player_mut(cur).hand.pop() {
                                    extra.push(c);
                                } else if let Some(c) = core.player_mut(cur).equips.pop() {
                                    core.declared_slots.remove(&c);
                                    crate::kernel::revoke_equip_skills(core, catalog, cur, c);
                                    extra.push(c);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(mark) = self.mark_cost {
            if let Some(v) = core.vm_aux.marks.get_mut(&(owner, mark)) {
                *v = v.saturating_sub(1);
            }
        }
        if let Some(cost) = self.cost {
            match cost {
                VmViewAsCost::TurnOver => {
                    let fd = core.player(owner).face_down;
                    core.player_mut(owner).face_down = !fd;
                }
                VmViewAsCost::GainMaxHp1 => {
                    core.player_mut(owner).max_hp = core.player(owner).max_hp.saturating_add(1);
                }
                VmViewAsCost::LoseMaxHp1 => {
                    if core.player(owner).max_hp > 1 {
                        core.player_mut(owner).max_hp -= 1;
                        let max_hp = core.player(owner).max_hp as i32;
                        if core.player(owner).hp > max_hp {
                            core.player_mut(owner).hp = max_hp;
                        }
                    }
                }
                VmViewAsCost::DisableEquipSlot => {
                    let mut mask = core
                        .vm_aux
                        .flags
                        .get(&(owner, "disabled_equip_slots".to_string()))
                        .copied()
                        .unwrap_or(0);
                    for s in 1..=5u32 {
                        if (mask & (1u32 << s)) == 0 {
                            mask |= 1u32 << s;
                            core.vm_aux
                                .flags
                                .insert((owner, "disabled_equip_slots".to_string()), mask);
                            core.vm_aux
                                .flags
                                .insert((owner, format!("equip_slot_disabled:{s}")), 1);
                            break;
                        }
                    }
                }
                VmViewAsCost::DrawOneOutsideTurnEmptyHand => {
                    if let Some(c) = core.deck.pop() {
                        core.player_mut(owner).hand.push(c);
                    }
                }
                VmViewAsCost::AllHand => {
                    let rest: Vec<CardId> = core.player_mut(owner).hand.drain(..).collect();
                    extra.extend(rest);
                }
                VmViewAsCost::FaceDownHand
                | VmViewAsCost::Guhuo
                | VmViewAsCost::Dczhaowen
                | VmViewAsCost::SwapLowerHand
                | VmViewAsCost::HandEquipMark => {
                    if !core.player(owner).hand.is_empty() {
                        extra.push(core.player_mut(owner).hand.remove(0));
                    } else if !core.player(owner).equips.is_empty() {
                        extra.push(core.player_mut(owner).equips.remove(0));
                    }
                }
                _ => {}
            }
        }
        if self.once_per_round {
            let key = if let Some(b) = self.branch {
                format!("view_as_round:{}:{b}", self.skill_name)
            } else {
                format!("view_as_round:{}", self.skill_name)
            };
            core.vm_aux.once.insert((owner, key, 2));
        }
        if self.once_per_turn_per_branch {
            let b = self.branch.unwrap_or(self.as_kind as u8);
            let key = format!("view_as_branch_turn:{}:{b}", self.skill_name);
            core.vm_aux.once.insert((owner, key, 1));
        }
        if let Some(nb) = self.next_branch {
            core.vm_aux.flags.insert(
                (owner, format!("view_as_branch:{}", self.skill_name)),
                u32::from(nb),
            );
        }
        self.record_use(core, owner);
        extra
    }

    pub fn record_use(&self, core: &mut GameStateCore, owner: PlayerId) {
        if self.zone == VmViewAsZone::None && self.skill_name == "dddxiaxing" {
            core.vm_aux
                .flags
                .remove(&(owner, "dddxiaxing_xia".to_string()));
        }
        core.vm_aux
            .flags
            .insert((owner, "event_view_as_skill".to_string()), self.skill_def_id);
        if self.uses_per_turn.is_some() {
            core.vm_aux
                .once
                .insert((owner, format!("view_as:{}", self.skill_name), 1));
        }
        if self.post_draw > 0 {
            for _ in 0..self.post_draw {
                if let Some(c) = core.deck.pop() {
                    core.player_mut(owner).hand.push(c);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmTriggerProgram {
    pub program: Arc<VmSkillProgram>,
    pub meta: TriggerMeta,
    pub filter: VmFilterPolicy,
    pub choice_actor: VmChoiceActor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmHandlerSet {
    pub skill_def_id: u32,
    pub active: Option<Arc<VmSkillProgram>>,
    pub limits: VmActiveLimits,
    pub triggers: Vec<VmTriggerProgram>,
    pub modifiers: Vec<VmModifierRule>,
    pub view_as: Vec<VmViewAsSpec>,
}

impl VmHandlerSet {
    pub(super) fn available(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
        let p = ctx.state.player(owner);
        let Some(active) = &self.active else {
            return false;
        };
        let name = &active.skill_name;
        let base_ok = p.alive
            && ctx.state.turn.current == owner
            && ctx.state.turn.phase == crate::event::Phase::Play
            && ctx
                .state
                .vm_aux
                .flags
                .get(&(owner, format!("skill_banned_turn:{name}")))
                .copied()
                .unwrap_or(0)
                == 0
            && (!self.limits.once_per_game || ctx.state.vm_aux.once_count(owner, 3, name) == 0)
            && (!self.limits.once_per_turn || ctx.state.vm_aux.once_count(owner, 1, name) == 0)
            && (!self.limits.once_per_play_phase
                || ctx.state.vm_aux.once_count(owner, 4, name) == 0)
            && self
                .limits
                .uses_per_turn
                .is_none_or(|max| ctx.state.vm_aux.once_count(owner, 1, name) < max as usize)
            && self
                .limits
                .uses_per_play_phase
                .is_none_or(|max| ctx.state.vm_aux.once_count(owner, 4, name) < max as usize)
            && self.limits.uses_per_turn_source.is_none_or(|src| {
                let cap = src.evaluate(ctx.state, owner);
                cap > 0 && ctx.state.vm_aux.once_count(owner, 1, name) < cap
            })
            && (!self.limits.hand_nonempty || !p.hand.is_empty())
            && (!self.limits.equip_nonempty || !p.equips.is_empty())
            && (!self.limits.hand_gt_hp || p.hand.len() as i64 > p.hp as i64)
            && (!self.limits.hand_gt_max_hp || p.hand.len() > p.max_hp as usize)
            && (!self.limits.face_up || !p.face_down)
            && (!self.limits.owner_unlinked || !p.linked)
            && (self.limits.require_missing_skill == 0
                || !owner_has_skill_hash(
                    ctx.state,
                    ctx.catalog,
                    owner,
                    self.limits.require_missing_skill,
                ))
            && (!self.limits.require_hand_limit_positive
                || ctx.state.hand_limit(ctx.catalog, owner) > 0)
            && (self.limits.require_hand_missing_gaintag == 0
                || (ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(
                        owner,
                        format!("gaintag:{}", self.limits.require_hand_missing_gaintag),
                    ))
                    .copied()
                    .unwrap_or(0)
                    == 0
                    && ctx
                        .state
                        .vm_aux
                        .marks
                        .get(&(owner, self.limits.require_hand_missing_gaintag))
                        .copied()
                        .unwrap_or(0)
                        <= 0))
            && (!self.limits.target_other
                || ctx.state.players.iter().enumerate().any(|(idx, q)| {
                    idx as u8 != owner
                        && q.alive
                        && self
                            .limits
                            .target_max_hp_gt
                            .is_none_or(|min_exclusive| q.max_hp > min_exclusive as u32)
                }))
            && self.limits.mark_at_least.is_none_or(|(mark, count)| {
                ctx.state
                    .vm_aux
                    .marks
                    .get(&(owner, mark))
                    .copied()
                    .unwrap_or(0) as i64
                    >= count as i64
            });
        if !base_ok {
            return false;
        }
        let cand_cards = self
            .limits
            .selection
            .map(|sel| sel.candidate_cards(ctx.state, ctx.catalog, owner, name));
        if let (Some(sel), Some(cards)) = (self.limits.selection, &cand_cards) {
            if cards.len() < sel.min_cards as usize {
                return false;
            }
        }
        if let Some(tgts) = self.limits.targets {
            let cand_targets = tgts.candidate_targets(ctx.state, ctx.catalog, owner, name);
            if cand_targets.len() < tgts.min_targets as usize {
                return false;
            }
            if let (Some(sel), Some(cards)) = (self.limits.selection, &cand_cards) {
                let max_cards_avail = cards.len().min(sel.max_cards as usize);
                if tgts.match_selected_card_count {
                    let min_req = sel.min_cards.max(tgts.min_targets) as usize;
                    let max_avail = max_cards_avail
                        .min(cand_targets.len())
                        .min(tgts.effective_max_targets(ctx.state, owner));
                    if max_avail < min_req {
                        return false;
                    }
                }
                if tgts.selected_cards_match_target_equip_count
                    && !cand_targets.iter().any(|&q| {
                        let eq_len = ctx.state.player(q).equips.len();
                        eq_len >= sel.min_cards as usize && eq_len <= max_cards_avail
                    })
                {
                    return false;
                }
            }
        }
        true
    }

    pub(super) fn matching<'a>(
        &'a self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> Vec<&'a VmTriggerProgram> {
        self.triggers
            .iter()
            .filter(|entry| {
                (entry.meta.event == kind || entry.meta.extra_events.contains(&kind))
                    && VmSkillBehavior {
                        program: entry.program.clone(),
                        filter: entry.filter.clone(),
                    }
                    .filter(ctx, owner, event)
            })
            .collect()
    }
}

pub(super) fn machine(
    program: Arc<VmSkillProgram>,
    filter: &VmFilterPolicy,
    owner: PlayerId,
    event: &EventData,
) -> VmMachine {
    fn named_meta(f: &VmFilterPolicy, skill_name: &str) -> (Vec<(String, u8)>, bool) {
        match f {
            VmFilterPolicy::Named(p) => (p.consume.clone(), p.count_hits),
            VmFilterPolicy::GameStarted { inner } => {
                let (mut c, h) = named_meta(inner, skill_name);
                c.push((format!("game_started:{skill_name}"), 3));
                (c, h)
            }
            VmFilterPolicy::Handlers(set) => {
                let mut c = Vec::new();
                if set.limits.once_per_game {
                    c.push((String::new(), 3));
                }
                if set.limits.once_per_turn
                    || set.limits.uses_per_turn.is_some()
                    || set.limits.uses_per_turn_source.is_some()
                {
                    c.push((String::new(), 1));
                }
                if set.limits.once_per_play_phase || set.limits.uses_per_play_phase.is_some() {
                    c.push((String::new(), 4));
                }
                if set.limits.target_other {
                    c.push((
                        "__target_other".to_string(),
                        set.limits.target_max_hp_gt.unwrap_or(0),
                    ));
                }
                (c, false)
            }
            VmFilterPolicy::AtPhase { inner, .. }
            | VmFilterPolicy::DrawPhaseSkipped { inner }
            | VmFilterPolicy::PhaseAnySkipped { inner }
            | VmFilterPolicy::Declared { inner, .. } => named_meta(inner, skill_name),
            _ => (Vec::new(), false),
        }
    }
    let (consume, count_hits) = named_meta(filter, &program.skill_name);
    VmMachine {
        owner,
        program,
        trig: trigger_ctx_of(event),
        event: event.clone(),
        pending_effects: Vec::new(),
        paused: None,
        asked: false,
        settled: false,
        consume,
        count_hits,
        commit_pause: None,
        following: Vec::new(),
        commit_state: Vec::new(),
        committing: false,
    }
}
