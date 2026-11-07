//! 主动技选牌、选目标及每回合动态次数约束规格。

use super::*;
use crate::catalog::{Catalog, EquipSlot, Suit};
use crate::state::{GameMode, GameStateCore, Role};
use crate::vocabulary::generals::Gender;

/// 主动技每回合可用次数动态来源（对齐 `content-schema::PackageSkillEntry::uses_per_turn_source`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmTurnUseSource {
    OwnerHp,
    OwnerLostHpMinOne,
    LivingRebelCount,
}

impl VmTurnUseSource {
    pub fn evaluate(self, state: &GameStateCore, owner: PlayerId) -> usize {
        let p = state.player(owner);
        match self {
            Self::OwnerHp => p.hp.max(0) as usize,
            Self::OwnerLostHpMinOne => (p.max_hp as i32 - p.hp).max(1) as usize,
            Self::LivingRebelCount => match &state.mode {
                GameMode::Identity { roles } => roles
                    .iter()
                    .enumerate()
                    .filter(|(idx, r)| **r == Role::Rebel && state.player(*idx as u8).alive)
                    .count(),
                _ => state.alive_players().filter(|&q| q != owner).count(),
            },
        }
    }
}

/// 主动技选牌区域（对齐 `content-schema::PackageSelectionSpec::zone`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VmSelectionZone {
    #[default]
    Hand,
    Equip,
    HandEquip,
}

/// 主动技选牌过滤器（对齐 `content-schema::PackageSelectionSpec::filter`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmSelectionCardFilter {
    CardEquip,
    CardArmor,
    CardWeapon,
    CardMount,
    CardTreasure,
    CardSha,
    CardShan,
    CardTao,
    CardJiu,
    CardBasic,
    CardNonBasic,
    CardNonEquip,
    CardTrick,
    CardDamageTrick,
    CardOrdinaryTrick,
    CardDelayedTrick,
    CardRed,
    CardBlack,
    CardSpade,
    CardHeart,
    CardClub,
    CardDiamond,
}

impl VmSelectionCardFilter {
    pub fn matches(self, catalog: &Catalog, card: CardId) -> bool {
        let Some(def) = catalog.cards.get(card as usize) else {
            return false;
        };
        let kind = def.kind;
        let suit = def.suit;
        let is_trick = !kind.is_basic_kind() && !kind.is_equip();
        let is_delayed = matches!(
            kind,
            crate::catalog::CardKind::Le
                | crate::catalog::CardKind::Bing
                | crate::catalog::CardKind::ShanDian
                | crate::catalog::CardKind::HongShui
        );
        match self {
            Self::CardEquip => kind.is_equip(),
            Self::CardArmor => kind.equip_slot() == Some(EquipSlot::Armor),
            Self::CardWeapon => kind.equip_slot() == Some(EquipSlot::Weapon),
            Self::CardMount => matches!(
                kind.equip_slot(),
                Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
            ),
            Self::CardTreasure => kind.equip_slot() == Some(EquipSlot::Treasure),
            Self::CardSha => kind.is_sha_like(),
            Self::CardShan => kind == crate::catalog::CardKind::Shan,
            Self::CardTao => kind == crate::catalog::CardKind::Tao,
            Self::CardJiu => kind == crate::catalog::CardKind::Jiu,
            Self::CardBasic => kind.is_basic_kind(),
            Self::CardNonBasic => !kind.is_basic_kind(),
            Self::CardNonEquip => !kind.is_equip(),
            Self::CardTrick => is_trick,
            Self::CardDamageTrick => matches!(
                kind,
                crate::catalog::CardKind::Duel
                    | crate::catalog::CardKind::SavageAttack
                    | crate::catalog::CardKind::ArcheryAttack
                    | crate::catalog::CardKind::ShanDian
                    | crate::catalog::CardKind::HongShui
                    | crate::catalog::CardKind::Linghunzhihuo
                    | crate::catalog::CardKind::Shenhuofeiya
                    | crate::catalog::CardKind::Shatang
                    | crate::catalog::CardKind::Yuansuhuimie
                    | crate::catalog::CardKind::Aozuzhilei
                    | crate::catalog::CardKind::Jingleishan
                    | crate::catalog::CardKind::Chiyuxi
                    | crate::catalog::CardKind::Liuxinghuoyu
            ),
            Self::CardOrdinaryTrick => is_trick && !is_delayed,
            Self::CardDelayedTrick => is_delayed,
            Self::CardRed => matches!(suit, Suit::Heart | Suit::Diamond),
            Self::CardBlack => matches!(suit, Suit::Spade | Suit::Club),
            Self::CardSpade => suit == Suit::Spade,
            Self::CardHeart => suit == Suit::Heart,
            Self::CardClub => suit == Suit::Club,
            Self::CardDiamond => suit == Suit::Diamond,
        }
    }
}

/// 主动技选牌规格（对齐 `content-schema::PackageSelectionSpec`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmActiveSelection {
    pub min_cards: u8,
    pub max_cards: u8,
    pub zone: VmSelectionZone,
    pub filter: Option<VmSelectionCardFilter>,
    pub unique_category_in_turn: bool,
    pub unused_category_by_skill_this_play_phase: bool,
    pub unused_category_by_owner_history_this_turn: bool,
    pub same_suit: bool,
    pub same_color: bool,
    pub distinct_suits: bool,
    pub distinct_categories: bool,
    pub sum_effective_number_exact: Option<u8>,
}

fn find_sum_subset(
    cards: &[CardId],
    catalog: &Catalog,
    idx: usize,
    min_len: usize,
    max_len: usize,
    rem_sum: u32,
    cur: &mut Vec<CardId>,
) -> bool {
    if cur.len() >= min_len && cur.len() <= max_len && rem_sum == 0 {
        return true;
    }
    if cur.len() >= max_len || idx >= cards.len() || rem_sum == 0 {
        return false;
    }
    for i in idx..cards.len() {
        let c = cards[i];
        let rank = catalog.cards.get(c as usize).map_or(0, |d| u32::from(d.rank));
        if rank > 0 && rank <= rem_sum {
            cur.push(c);
            if find_sum_subset(cards, catalog, i + 1, min_len, max_len, rem_sum - rank, cur) {
                return true;
            }
            cur.pop();
        }
    }
    false
}

impl VmActiveSelection {
    pub fn card_category_bit(catalog: &Catalog, card: CardId) -> u32 {
        let Some(def) = catalog.cards.get(card as usize) else {
            return 0;
        };
        let k = def.kind;
        if k.is_equip() {
            4
        } else if k.is_basic_kind() {
            1
        } else {
            2
        }
    }

    pub fn candidate_cards(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_name: &str,
    ) -> Vec<CardId> {
        let p = state.player(owner);
        let mut used_mask = 0u32;
        if self.unique_category_in_turn {
            used_mask |= state
                .vm_aux
                .flags
                .get(&(owner, format!("unique_cat_mask:{skill_name}")))
                .or_else(|| state.vm_aux.flags.get(&(owner, "unique_cat_mask".to_string())))
                .copied()
                .unwrap_or(0);
        }
        if self.unused_category_by_skill_this_play_phase {
            used_mask |= state
                .vm_aux
                .flags
                .get(&(owner, format!("unique_cat_play_mask:{skill_name}")))
                .copied()
                .unwrap_or(0);
            for bit in [1u32, 2, 4] {
                if state
                    .vm_aux
                    .once
                    .contains(&(owner, format!("unique_cat_play:{skill_name}:{bit}"), 4))
                {
                    used_mask |= bit;
                }
            }
        }
        if self.unused_category_by_owner_history_this_turn {
            used_mask |= state
                .vm_aux
                .flags
                .get(&(owner, "play_cat_mask".to_string()))
                .copied()
                .unwrap_or(0);
        }
        let mut raw = Vec::new();
        let iter_hand = matches!(self.zone, VmSelectionZone::Hand | VmSelectionZone::HandEquip);
        let iter_equips = matches!(self.zone, VmSelectionZone::Equip | VmSelectionZone::HandEquip);
        for &c in (if iter_hand { p.hand.as_slice() } else { &[] })
            .iter()
            .chain(if iter_equips { p.equips.as_slice() } else { &[] })
        {
            if !self.filter.map_or(true, |f| f.matches(catalog, c)) {
                continue;
            }
            if used_mask != 0 && (Self::card_category_bit(catalog, c) & used_mask) != 0 {
                continue;
            }
            raw.push(c);
        }
        let min_c = self.min_cards as usize;
        let max_c = self.max_cards as usize;
        if let Some(exact_sum) = self.sum_effective_number_exact {
            let mut chosen = Vec::new();
            if !find_sum_subset(
                &raw,
                catalog,
                0,
                min_c.max(1),
                max_c.max(1),
                u32::from(exact_sum),
                &mut chosen,
            ) {
                return Vec::new();
            }
            return chosen;
        }
        if self.same_suit && min_c > 1 {
            for suit in [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond] {
                let matching: Vec<CardId> = raw
                    .iter()
                    .copied()
                    .filter(|&c| catalog.cards.get(c as usize).is_some_and(|d| d.suit == suit))
                    .collect();
                if matching.len() >= min_c {
                    return matching;
                }
            }
            return Vec::new();
        }
        if self.same_color && min_c > 1 {
            for want_red in [true, false] {
                let matching: Vec<CardId> = raw
                    .iter()
                    .copied()
                    .filter(|&c| {
                        catalog.cards.get(c as usize).is_some_and(|d| {
                            matches!(d.suit, Suit::Heart | Suit::Diamond) == want_red
                        })
                    })
                    .collect();
                if matching.len() >= min_c {
                    return matching;
                }
            }
            return Vec::new();
        }
        if self.distinct_suits && min_c > 1 {
            let mut distinct = Vec::new();
            let mut seen_suits = 0u8;
            for &c in &raw {
                if let Some(def) = catalog.cards.get(c as usize) {
                    let bit = 1u8 << (def.suit as u8);
                    if (seen_suits & bit) == 0 {
                        seen_suits |= bit;
                        distinct.push(c);
                    }
                }
            }
            if distinct.len() < min_c {
                return Vec::new();
            }
            return distinct;
        }
        if self.distinct_categories && min_c > 1 {
            let mut distinct = Vec::new();
            let mut seen_cats = 0u32;
            for &c in &raw {
                let bit = Self::card_category_bit(catalog, c);
                if bit != 0 && (seen_cats & bit) == 0 {
                    seen_cats |= bit;
                    distinct.push(c);
                }
            }
            if distinct.len() < min_c {
                return Vec::new();
            }
            return distinct;
        }
        raw
    }
}

/// 主动技选目标过滤器（对齐 `content-schema::PackageTargetSpec::filter`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmTargetFilter {
    pub other_only: bool,
    pub wounded_only: bool,
    pub hand_nonempty: bool,
    pub equip_nonempty: bool,
    pub hand_equip_nonempty: bool,
    pub hand_equip_judge_nonempty: bool,
    pub within_attack_range: bool,
    pub owner_in_target_attack_range: bool,
    pub hand_gt_owner: bool,
    pub hand_ge_owner: bool,
    pub hand_le_owner: bool,
    pub hand_lt: Option<u8>,
    pub hp_ge_owner: bool,
    pub hp_gt_owner: bool,
    pub hp_lte_owner: bool,
    pub hp_gt_zero: bool,
    pub hp_gt: Option<u8>,
    pub max_hp_gt: Option<u8>,
    pub lowest_hp: bool,
    pub highest_hp: bool,
    pub has_discardable_equip: bool,
    pub has_gainable_equip: bool,
    pub has_gainable_hand_equip: bool,
    pub has_discardable_hand_equip: bool,
    pub can_use_on_owner: bool,
    pub can_use_sha_on_owner: bool,
    pub selected_cards_match_target_equip_count: bool,
    pub same_group_with_owner: bool,
    pub different_group_from_owner: bool,
    pub male_only: bool,
    pub female_only: bool,
    pub linked_only: bool,
    pub unlinked_only: bool,
    pub unique_in_turn: bool,
    pub unique_in_play_phase: bool,
}

impl VmTargetFilter {
    pub fn matches(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        target: PlayerId,
        skill_name: &str,
    ) -> bool {
        let tp = state.player(target);
        if !tp.alive {
            return false;
        }
        if self.other_only && target == owner {
            return false;
        }
        let op = state.player(owner);
        if self.wounded_only && tp.hp >= tp.max_hp as i32 {
            return false;
        }
        if self.hand_nonempty && tp.hand.is_empty() {
            return false;
        }
        if (self.equip_nonempty
            || self.has_discardable_equip
            || self.has_gainable_equip
            || self.selected_cards_match_target_equip_count)
            && tp.equips.is_empty()
        {
            return false;
        }
        if (self.hand_equip_nonempty
            || self.has_gainable_hand_equip
            || self.has_discardable_hand_equip)
            && tp.hand.is_empty()
            && tp.equips.is_empty()
        {
            return false;
        }
        if self.hand_equip_judge_nonempty
            && tp.hand.is_empty()
            && tp.equips.is_empty()
            && tp.judge.is_empty()
        {
            return false;
        }
        if self.within_attack_range
            && (target == owner
                || state.computed_distance(catalog, owner, target)
                    > state.attack_range(catalog, owner))
        {
            return false;
        }
        if (self.owner_in_target_attack_range || self.can_use_sha_on_owner)
            && (target == owner
                || state.computed_distance(catalog, target, owner)
                    > state.attack_range(catalog, target))
        {
            return false;
        }
        if self.can_use_on_owner && target == owner {
            return false;
        }
        if self.hand_gt_owner && tp.hand.len() <= op.hand.len() {
            return false;
        }
        if self.hand_ge_owner && tp.hand.len() < op.hand.len() {
            return false;
        }
        if self.hand_le_owner && tp.hand.len() > op.hand.len() {
            return false;
        }
        if let Some(lim) = self.hand_lt {
            if tp.hand.len() >= lim as usize {
                return false;
            }
        }
        if self.hp_ge_owner && tp.hp < op.hp {
            return false;
        }
        if self.hp_gt_owner && tp.hp <= op.hp {
            return false;
        }
        if self.hp_lte_owner && tp.hp > op.hp {
            return false;
        }
        if self.hp_gt_zero && tp.hp <= 0 {
            return false;
        }
        if let Some(min_hp) = self.hp_gt {
            if tp.hp <= i32::from(min_hp) {
                return false;
            }
        }
        if let Some(min_ex) = self.max_hp_gt {
            if tp.max_hp <= u32::from(min_ex) {
                return false;
            }
        }
        if self.lowest_hp {
            let min_hp = state
                .alive_players()
                .map(|q| state.player(q).hp)
                .min()
                .unwrap_or(tp.hp);
            if tp.hp != min_hp {
                return false;
            }
        }
        if self.highest_hp {
            let max_hp = state
                .alive_players()
                .map(|q| state.player(q).hp)
                .max()
                .unwrap_or(tp.hp);
            if tp.hp != max_hp {
                return false;
            }
        }
        if self.same_group_with_owner
            && state.effective_faction(catalog, owner) != state.effective_faction(catalog, target)
        {
            return false;
        }
        if self.different_group_from_owner
            && state.effective_faction(catalog, owner) == state.effective_faction(catalog, target)
        {
            return false;
        }
        if self.male_only && state.effective_gender(target) != Gender::Male {
            return false;
        }
        if self.female_only && state.effective_gender(target) != Gender::Female {
            return false;
        }
        if self.linked_only && !tp.linked {
            return false;
        }
        if self.unlinked_only && tp.linked {
            return false;
        }
        let target_raw = u32::from(target) + 1;
        if self.unique_in_turn {
            let rec = state
                .vm_aux
                .flags
                .get(&(owner, format!("unique_cat_target:{skill_name}")))
                .or_else(|| state.vm_aux.flags.get(&(owner, "unique_cat_target".to_string())))
                .copied()
                .unwrap_or(0);
            if rec == target_raw
                || state
                    .vm_aux
                    .once
                    .contains(&(owner, format!("active_target:{skill_name}:{target}"), 1))
            {
                return false;
            }
        }
        if self.unique_in_play_phase {
            let rec = state
                .vm_aux
                .flags
                .get(&(owner, format!("active_target:{skill_name}")))
                .or_else(|| state.vm_aux.flags.get(&(owner, "active_target".to_string())))
                .copied()
                .unwrap_or(0);
            if rec == target_raw
                || state
                    .vm_aux
                    .once
                    .contains(&(owner, format!("active_target:{skill_name}:{target}"), 4))
            {
                return false;
            }
        }
        true
    }
}

/// 主动技选目标规格（对齐 `content-schema::PackageTargetSpec`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmActiveTargets {
    pub min_targets: u8,
    pub max_targets: u8,
    pub filter: VmTargetFilter,
    pub max_by_owner_mark: Option<u32>,
    pub max_by_owner_lost_hp: bool,
    pub max_by_owner_hp: bool,
    pub match_selected_card_count: bool,
    pub selected_cards_match_target_equip_count: bool,
    pub pair_hand_difference_lte_three: bool,
    pub second_can_use_juedou_on_first: bool,
    pub equipment_pair_with_armor: bool,
}

impl VmActiveTargets {
    pub fn effective_max_targets(&self, state: &GameStateCore, owner: PlayerId) -> usize {
        let mut cap = self.max_targets as usize;
        if let Some(mark) = self.max_by_owner_mark {
            let m = state
                .vm_aux
                .marks
                .get(&(owner, mark))
                .copied()
                .unwrap_or(0)
                .max(0) as usize;
            cap = cap.min(m);
        }
        if self.max_by_owner_lost_hp {
            let p = state.player(owner);
            let lost = (p.max_hp as i32 - p.hp).max(0) as usize;
            cap = cap.min(lost);
        }
        if self.max_by_owner_hp {
            let hp = state.player(owner).hp.max(0) as usize;
            cap = cap.min(hp);
        }
        cap
    }

    pub fn candidate_targets(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_name: &str,
    ) -> Vec<PlayerId> {
        let eff_max = self.effective_max_targets(state, owner);
        if self.min_targets > 0 && eff_max < self.min_targets as usize {
            return Vec::new();
        }
        let n = state.players.len();
        let mut out = Vec::new();
        for step in 0..n {
            let q = ((owner as usize + step) % n) as PlayerId;
            if self.filter.matches(state, catalog, owner, q, skill_name) {
                out.push(q);
            }
        }
        if self.pair_hand_difference_lte_three && self.min_targets >= 2 {
            let mut found = None;
            'pair_diff: for (i, &a) in out.iter().enumerate() {
                for &b in &out[i + 1..] {
                    if state.player(a).hand.len().abs_diff(state.player(b).hand.len()) <= 3 {
                        found = Some((a, b));
                        break 'pair_diff;
                    }
                }
            }
            let Some((a, b)) = found else {
                return Vec::new();
            };
            let mut reordered = vec![a, b];
            for q in out {
                if q != a && q != b {
                    reordered.push(q);
                }
            }
            out = reordered;
        }
        if self.equipment_pair_with_armor && self.min_targets >= 2 {
            let has_armor = |p: PlayerId| {
                state
                    .player(p)
                    .equips
                    .iter()
                    .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Armor))
            };
            let mut found = None;
            'pair_armor: for (i, &a) in out.iter().enumerate() {
                if state.player(a).equips.is_empty() {
                    continue;
                }
                for &b in &out[i + 1..] {
                    if !state.player(b).equips.is_empty() && (has_armor(a) || has_armor(b)) {
                        found = Some((a, b));
                        break 'pair_armor;
                    }
                }
            }
            let Some((a, b)) = found else {
                return Vec::new();
            };
            let mut reordered = vec![a, b];
            for q in out {
                if q != a && q != b {
                    reordered.push(q);
                }
            }
            out = reordered;
        }
        if self.second_can_use_juedou_on_first && self.min_targets >= 2 && out.len() < 2 {
            return Vec::new();
        }
        out
    }
}

/// 可选触发的确认决策角色（对齐 `content-schema::PackageTriggerEntry::choice_actor`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VmChoiceActor {
    #[default]
    Owner,
    EventPlayer,
    EventSource,
}

impl VmChoiceActor {
    pub fn resolve(self, owner: PlayerId, event: &EventData) -> PlayerId {
        match self {
            Self::Owner => owner,
            Self::EventPlayer => trigger_ctx_of(event).event_player.unwrap_or(owner),
            Self::EventSource => trigger_ctx_of(event).event_source.unwrap_or(owner),
        }
    }
}
