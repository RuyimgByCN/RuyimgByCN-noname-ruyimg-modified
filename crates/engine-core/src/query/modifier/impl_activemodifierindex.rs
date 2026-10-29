//! Query operations for `matches_kind_and_suit` and related helpers.

use super::*;

impl ActiveModifierIndex {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Len.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Clear.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Register skill modifiers (catalog from content registry).
    pub fn register_skill(
        &mut self,
        skill_id: SkillId,
        skill_def: SkillDefId,
        owner: PlayerId,
        content: &ContentRegistry,
    ) {
        for d in content.modifier_defs_for_skill(skill_def) {
            self.entries.push(ActiveModifier {
                skill_id,
                skill_def,
                owner,
                def_id: d.id,
                target: d.target,
                mode: d.mode,
                value_source: d.value_source,
                value: d.value,
                order: d.order,
                enabled: true,
                label_key: d.label_key,
            });
        }
        self.entries.sort_by_key(|e| (e.skill_id.raw(), e.def_id));
    }

    /// Unregister.
    pub fn unregister_skill(&mut self, skill_id: SkillId) {
        self.entries.retain(|e| e.skill_id != skill_id);
    }

    /// Set enabled flag for all rows of a skill instance.
    pub fn set_skill_enabled(&mut self, skill_id: SkillId, enabled: bool) {
        for entry in &mut self.entries {
            if entry.skill_id == skill_id {
                entry.enabled = enabled;
            }
        }
    }

    /// Rebuild.
    pub fn rebuild_from<'a, I>(&mut self, skills: I, content: &ContentRegistry)
    where
        I: IntoIterator<Item = (SkillId, SkillDefId, PlayerId, bool)>,
    {
        self.clear();
        for (sid, sdef, owner, enabled) in skills {
            for d in content.modifier_defs_for_skill(sdef) {
                self.entries.push(ActiveModifier {
                    skill_id: sid,
                    skill_def: sdef,
                    owner,
                    def_id: d.id,
                    target: d.target,
                    mode: d.mode,
                    value_source: d.value_source,
                    value: d.value,
                    order: d.order,
                    enabled,
                    label_key: d.label_key,
                });
            }
        }
        self.entries.sort_by_key(|e| (e.skill_id.raw(), e.def_id));
    }

    /// Collect modifiers for a target channel, stable order.
    pub fn for_target(&self, target: ModifierTarget) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|e| e.enabled && e.target == target)
            .collect();
        // Combine order: mode class then order then skill_id
        v.sort_by_key(|e| (mode_rank(e.mode), e.order, e.skill_id.raw(), e.def_id));
        v
    }

    /// First configured mark cost for this player's enabled judgment rewrite.
    pub fn judgment_rewrite_cost_mark(&self, player: PlayerId) -> Option<u32> {
        self.entries.iter().find_map(|entry| match entry.target {
            ModifierTarget::JudgmentRewriteHandEquipCostMark(mark)
                if entry.enabled && entry.owner == player =>
            {
                Some(mark)
            }
            _ => None,
        })
    }

    /// Collect owner-local and living-owner global attack-range modifiers in one stable order.
    pub fn for_attack_range(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::AttackRange => entry.owner == player,
                        ModifierTarget::GlobalAttackRange => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect owner-local and living-owner global hand-limit modifiers in stable order.
    pub fn for_hand_limit(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::HandLimit => entry.owner == player,
                        ModifierTarget::GlobalHandLimit => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect owner-local and living-owner global draw-count modifiers in stable order.
    pub fn for_draw_count(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DrawCount => entry.owner == player,
                        ModifierTarget::GlobalDrawCount => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect owner-local and living-owner global Sha-use-limit modifiers in stable order.
    pub fn for_sha_use_limit(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::ShaUseLimit => entry.owner == player,
                        ModifierTarget::GlobalShaUseLimit => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect active hand-limit exclusions matching one concrete card.
    pub fn for_hand_limit_card(
        &self,
        state: &GameState,
        player: PlayerId,
        card: &Card,
        kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::ShaIgnoredHandLimit => kind.is_sha(),
                        ModifierTarget::CardIgnoredHandLimit(selector) => {
                            if selector == CardSelector::SkillTracked {
                                entry.owner == player
                                    && state.skills.get(entry.skill_id).is_some_and(|skill| {
                                        skill.tracked_cards().contains(&card.id)
                                    })
                            } else {
                                selector.matches_physical_card(state, player, card, suit)
                            }
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect distance-limit exemptions matching one concrete card.
    pub fn for_card_distance(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardDistanceUnlimited => true,
                        ModifierTarget::CardDistanceUnlimitedFor(selector) => {
                            selector.matches_kind_and_suit(kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned discard protections matching one physical equipment card.
    pub fn for_card_discard_protection(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardDiscardProtectedFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned gain protections matching one physical card.
    pub fn for_card_gain_protection(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardGainProtectedFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-aware distance exemptions matching one concrete card.
    pub fn for_card_distance_if_target_hand_gte_owner(
        &self,
        kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect ordinary use-limit exemptions matching one concrete card.
    pub fn for_card_use_limit(
        &self,
        physical_kind: CardKind,
        effective_kind: CardKind,
        effective_suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardUseUnlimited => true,
                        ModifierTarget::CardUseUnlimitedFor(selector) => {
                            selector.matches_card(physical_kind, effective_kind, effective_suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-count modifiers matching one concrete card use.
    pub fn for_card_target_limit(
        &self,
        physical_kind: CardKind,
        effective_kind: CardKind,
        effective_suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetLimitFor(selector)
                            if selector.matches_card(
                                physical_kind,
                                effective_kind,
                                effective_suit,
                            )
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect outgoing Sha response locks matching one concrete effective card.
    pub fn for_sha_unrespondable(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::ShaUnrespondableFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect source-owned response locks matching the current logical card.
    pub fn for_card_use_unrespondable(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardUseUnrespondableFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect generic and card-matching outgoing Sha damage modifiers.
    pub fn for_sha_damage(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::ShaDamage => true,
                        ModifierTarget::ShaDamageFor(selector) => {
                            selector.matches_kind_and_suit(kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect bounded distance-limit overrides matching one concrete card.
    pub fn for_card_distance_limit(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardDistanceLimitFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect rule-category overrides matching a physical card category and effective suit.
    pub fn for_card_name(
        &self,
        state: &GameState,
        player: PlayerId,
        card: &Card,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardName(selector)
                            if if selector == CardSelector::SkillTracked {
                                state.skills.get(entry.skill_id).is_some_and(|skill| {
                                    skill.tracked_cards().contains(&card.id)
                                })
                            } else {
                                selector.matches_physical_card(state, player, card, suit)
                            }
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect damage-nature overrides matching one physical card category and effective suit.
    pub fn for_card_nature(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardNatureFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect card-use cancellation modifiers matching a logical card kind.
    pub fn for_card_use(&self, logical_kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardUseCancel => true,
                        ModifierTarget::CardUseCancelFor(selector) => {
                            selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect response-cancellation modifiers matching one logical card kind and suit.
    pub fn for_card_response(&self, logical_kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardResponseCancel => true,
                        ModifierTarget::CardResponseCancelFor(selector) => {
                            selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-side cancellation modifiers matching a logical card kind.
    pub fn for_card_target(
        &self,
        user: PlayerId,
        target: PlayerId,
        logical_kind: CardKind,
        suit: Suit,
        from_current: bool,
        use_ban_pairs: &[(PlayerId, PlayerId)],
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardTargetCancelFor(selector) => {
                            selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        ModifierTarget::CardTargetCancelFromCurrentFor(selector) => {
                            from_current && selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        ModifierTarget::CardTargetCancelTurnBannedFor(selector) => {
                            user != target
                                && use_ban_pairs.contains(&(user, target))
                                && selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect armor-owned target cancellation modifiers matching a logical card kind.
    pub fn for_armor_card_target(
        &self,
        logical_kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::ArmorCardTargetCancelFor(selector)
                            if selector.matches_kind_and_suit(logical_kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect hand-count/armor-conditional target cancellation modifiers.
    pub fn for_card_target_if_other_user_hand_gte_target_and_armor_empty(
        &self,
        logical_kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(selector)
                            if selector.matches_kind_and_suit(logical_kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect distance-conditional target cancellation modifiers matching one logical card.
    pub fn for_card_target_if_other_user_distance_gt_one(
        &self,
        logical_kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetCancelIfOtherUserDistanceGtOneFor(selector)
                            if selector.matches_kind_and_suit(logical_kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned damage cancellation modifiers for the resolved nature.
    pub fn for_damage_taken_cancel(&self, nature: u8) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageTakenCancel => true,
                        ModifierTarget::DamageTakenCancelForNature(value) => value == nature,
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect source-owned damage amount modifiers for the original nature.
    pub fn for_damage_amount(&self, nature: u8, logical_sha: bool) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageAmount => true,
                        ModifierTarget::DamageAmountForNature(value) => value == nature,
                        ModifierTarget::DamageAmountForSha => logical_sha,
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned and living-owner global damage amount modifiers.
    pub fn for_damage_taken_amount(
        &self,
        nature: u8,
        target: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageTakenAmount => entry.owner == target,
                        ModifierTarget::DamageTakenAmountForNature(value) => {
                            entry.owner == target && value == nature
                        }
                        ModifierTarget::GlobalDamageTakenAmount => global_owner_alive(entry.owner),
                        ModifierTarget::GlobalDamageTakenAmountForNature(value) => {
                            value == nature && global_owner_alive(entry.owner)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect armor-owned damage amount modifiers for the resolved nature.
    pub fn for_armor_damage_taken_amount(&self, nature: u8) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::ArmorDamageTakenAmountForNature(value)
                            if value == nature
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned and living-owner global final damage amount modifiers.
    pub fn for_damage_taken_final_amount(
        &self,
        nature: u8,
        target: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageTakenFinalAmount => entry.owner == target,
                        ModifierTarget::DamageTakenFinalAmountForNature(value) => {
                            entry.owner == target && value == nature
                        }
                        ModifierTarget::GlobalDamageTakenFinalAmount => {
                            global_owner_alive(entry.owner)
                        }
                        ModifierTarget::GlobalDamageTakenFinalAmountForNature(value) => {
                            value == nature && global_owner_alive(entry.owner)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect forced-all-target modifiers matching one concrete card kind.
    pub fn for_card_targets_all(&self, selector: CardSelector) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetsAllFor(found) if found == selector
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }
}
