//! Runtime skill state for `tag` and related helpers.

use super::*;

impl ResolutionState {
    pub(crate) fn record_duel_damage(&mut self, card: CardId) {
        if card.is_some() && !self.duel_damage_cards.contains(&card) {
            self.duel_damage_cards.push(card);
            self.duel_damage_cards.sort_by_key(|card| card.raw());
        }
    }

    pub(crate) fn take_duel_damage(&mut self, card: CardId) -> bool {
        let Some(index) = self
            .duel_damage_cards
            .iter()
            .position(|tracked| *tracked == card)
        else {
            return false;
        };
        self.duel_damage_cards.remove(index);
        true
    }

    pub(crate) const SAVED_SHA_RESPONSE_FLAGS_MARKER: u32 = u32::MAX;

    /// Register one exact Sha card for a post-use no-damage action.
    pub fn register_sha_no_damage_draw(&mut self, card: CardId, skill: SkillId) {
        self.sha_no_damage_draw_skills
            .retain(|(saved, _)| *saved != card);
        self.sha_no_damage_draw_skills.push((card, skill));
        self.sha_no_damage_draw_skills
            .sort_by_key(|(saved, _)| saved.raw());
    }

    /// Register at most one replay for an exact source card.
    pub fn register_card_replay(&mut self, replay: PendingCardReplay) -> bool {
        if self
            .pending_card_replays
            .iter()
            .any(|pending| pending.source_card == replay.source_card)
        {
            return false;
        }
        self.pending_card_replays.push(replay);
        true
    }

    /// Register one logical card use until the shared post-resolution idle boundary.
    pub(crate) fn register_card_resolution(
        &mut self,
        source: PlayerId,
        card: CardId,
        kind: u8,
        targets: Vec<PlayerId>,
        source_was_hand: bool,
        originating_view_as_skill_def: u32,
    ) {
        let planned = self
            .planned_card_source_cards
            .iter()
            .position(|(saved_source, saved_card, _, _)| {
                *saved_source == source && *saved_card == card
            })
            .map(|index| self.planned_card_source_cards.remove(index));
        let source_cards = planned
            .as_ref()
            .map(|(_, _, cards, _)| cards.clone())
            .or_else(|| {
                self.sha_resolution_records
                    .iter()
                    .find(|record| record.source == source && record.card == card)
                    .filter(|record| !record.source_cards.is_empty())
                    .map(|record| record.source_cards.clone())
            })
            .unwrap_or_else(|| vec![card]);
        let all_sources_were_hand = planned
            .as_ref()
            .map_or(source_was_hand, |(_, _, _, all_hand)| *all_hand)
            && !source_cards.is_empty();
        self.pending_card_resolutions.push(PendingCardResolution {
            source,
            card,
            kind,
            targets,
            damaged_targets: Vec::new(),
            source_was_hand,
            all_sources_were_hand,
            response_color_lock_mask: 0,
            source_cards,
            originating_view_as_skill_def,
            skill_excluded_targets: Vec::new(),
        });
    }

    pub(crate) fn record_ddd_tongyu_rejector(
        &mut self,
        source: PlayerId,
        card: CardId,
        target: PlayerId,
    ) -> bool {
        let Some(record) = self
            .pending_card_resolutions
            .iter_mut()
            .rev()
            .find(|record| {
                record.source == source
                    && record.card == card
                    && record.originating_view_as_skill_def == 2_302_761_270
            })
        else {
            return false;
        };
        if !record.targets.contains(&target) {
            return false;
        }
        if !record.skill_excluded_targets.contains(&target) {
            record.skill_excluded_targets.push(target);
            record
                .skill_excluded_targets
                .sort_by_key(|player| player.raw());
        }
        true
    }

    pub(crate) fn ddd_tongyu_rejectors(
        &self,
        owner: PlayerId,
    ) -> impl Iterator<Item = PlayerId> + '_ {
        self.pending_card_resolutions
            .iter()
            .rev()
            .filter(move |record| {
                record.source == owner && record.originating_view_as_skill_def == 2_302_761_270
            })
            .flat_map(|record| record.skill_excluded_targets.iter().copied())
    }

    pub(crate) fn ddd_tongyu_extra_reveal(&self, source: PlayerId, card: CardId) -> usize {
        self.pending_card_resolutions
            .iter()
            .rev()
            .find(|record| {
                record.source == source
                    && record.card == card
                    && record.originating_view_as_skill_def == 2_302_761_270
            })
            .map_or(0, |record| record.source_cards.len())
    }

    /// Preserve all physical materials until the queued logical use begins.
    pub(crate) fn plan_card_source_cards(
        &mut self,
        source: PlayerId,
        card: CardId,
        cards: Vec<CardId>,
        all_sources_were_hand: bool,
    ) {
        self.planned_card_source_cards
            .retain(|(saved_source, saved_card, _, _)| {
                *saved_source != source || *saved_card != card
            });
        self.planned_card_source_cards
            .push((source, card, cards, all_sources_were_hand));
    }

    /// Whether a queued logical use has an explicit physical source plan.
    pub(crate) fn has_planned_card_source(&self, source: PlayerId, card: CardId) -> bool {
        self.planned_card_source_cards
            .iter()
            .any(|(saved_source, saved_card, _, _)| *saved_source == source && *saved_card == card)
    }

    /// Physical materials for the innermost matching logical card use.
    pub(crate) fn card_source_cards(&self, source: PlayerId, card: CardId) -> Vec<CardId> {
        self.pending_card_resolutions
            .iter()
            .rev()
            .find(|record| record.source == source && record.card == card)
            .map(|record| record.source_cards.clone())
            .unwrap_or_else(|| vec![card])
    }

    /// Physical materials for the innermost matching logical card use, independent of damage
    /// source rewrites.
    pub(crate) fn card_source_cards_for_card(&self, card: CardId) -> Vec<CardId> {
        self.pending_card_resolutions
            .iter()
            .rev()
            .find(|record| record.card == card)
            .map(|record| record.source_cards.clone())
            .unwrap_or_else(|| vec![card])
    }

    /// Merge a red/black response lock into the innermost matching logical card use.
    pub(crate) fn lock_card_response_colors(&mut self, card: CardId, color_mask: u8) {
        if let Some(record) = self
            .pending_card_resolutions
            .iter_mut()
            .rev()
            .find(|record| record.card == card)
        {
            record.response_color_lock_mask |= color_mask & 0b11;
        }
    }

    /// Record committed damage for the innermost matching logical card use.
    pub(crate) fn record_card_resolution_damage(&mut self, card: CardId, target: PlayerId) {
        let Some(record) = self
            .pending_card_resolutions
            .iter_mut()
            .rev()
            .find(|record| record.card == card)
        else {
            return;
        };
        if !record.damaged_targets.contains(&target) {
            record.damaged_targets.push(target);
        }
    }

    /// Consume the post-use no-damage action for one exact Sha card.
    pub fn take_sha_no_damage_draw(&mut self, card: CardId) -> Option<SkillId> {
        let index = self
            .sha_no_damage_draw_skills
            .iter()
            .position(|(saved, _)| *saved == card)?;
        Some(self.sha_no_damage_draw_skills.remove(index).1)
    }

    /// Register or refresh one exact logical Sha use before its response queue begins.
    pub fn register_sha_resolution(
        &mut self,
        source: PlayerId,
        card: CardId,
        targets: &[PlayerId],
        use_counted: bool,
    ) {
        if let Some(record) = self
            .sha_resolution_records
            .iter_mut()
            .find(|record| record.source == source && record.card == card)
        {
            record.targets = targets.to_vec();
            record.use_counted |= use_counted;
            return;
        }
        self.sha_resolution_records.push(ShaResolutionRecord {
            source,
            card,
            targets: targets.to_vec(),
            use_counted,
            use_refunded: false,
            damage_source_none: false,
            source_cards: Vec::new(),
            response_cards: Vec::new(),
            skill_targets: Vec::new(),
            backwater_targets: Vec::new(),
            dying_targets: Vec::new(),
        });
    }

    /// Mark one exact in-flight Sha so its damage has no player source.
    pub fn mark_sha_damage_source_none(&mut self, source: PlayerId, card: CardId) -> bool {
        let Some(record) = self
            .sha_resolution_records
            .iter_mut()
            .find(|record| record.source == source && record.card == card)
        else {
            return false;
        };
        record.damage_source_none = true;
        true
    }

    /// Resolve the actual damage source for one exact in-flight Sha.
    pub fn sha_damage_source(&self, source: PlayerId, card: CardId) -> PlayerId {
        self.sha_resolution_records
            .iter()
            .find(|record| record.source == source && record.card == card)
            .filter(|record| record.damage_source_none)
            .map_or(source, |_| PlayerId::NONE)
    }

    /// Recover the logical Sha user when damage attribution is intentionally source-less.
    pub fn logical_sha_source(&self, card: CardId) -> PlayerId {
        self.sha_resolution_records
            .iter()
            .find(|record| record.card == card && record.damage_source_none)
            .map_or(PlayerId::NONE, |record| record.source)
    }

    /// Preserve every physical source consumed by one exact logical Sha use.
    pub fn register_sha_source_cards(
        &mut self,
        source: PlayerId,
        card: CardId,
        source_cards: &[CardId],
    ) -> bool {
        let Some(record) = self
            .sha_resolution_records
            .iter_mut()
            .find(|record| record.source == source && record.card == card)
        else {
            return false;
        };
        record.source_cards = source_cards.to_vec();
        true
    }

    /// Physical cards consumed by one logical Sha, falling back to its carrier.
    pub fn sha_source_cards(&self, source: PlayerId, card: CardId) -> Vec<CardId> {
        self.sha_resolution_records
            .iter()
            .find(|record| record.source == source && record.card == card)
            .filter(|record| !record.source_cards.is_empty())
            .map(|record| record.source_cards.clone())
            .unwrap_or_else(|| vec![card])
    }

    /// Bind one originating skill and target to the exact Sha currently resolving.
    pub fn record_sha_skill_target(
        &mut self,
        source: PlayerId,
        card: CardId,
        skill: SkillId,
        target: PlayerId,
    ) -> bool {
        let Some(record) = self
            .sha_resolution_records
            .iter_mut()
            .find(|record| record.source == source && record.card == card)
        else {
            return false;
        };
        if !record.targets.contains(&target) {
            return false;
        }
        if !record.skill_targets.contains(&(skill, target)) {
            record.skill_targets.push((skill, target));
            record
                .skill_targets
                .sort_by_key(|(skill, target)| (skill.raw(), target.raw()));
        }
        true
    }

    /// Whether one exact resolving Sha carries this skill/target binding.
    pub fn sha_skill_target_recorded(
        &self,
        source: PlayerId,
        card: CardId,
        skill: SkillId,
        target: PlayerId,
    ) -> bool {
        self.sha_resolution_records.iter().any(|record| {
            record.source == source
                && record.card == card
                && record.skill_targets.contains(&(skill, target))
        })
    }

    /// Arm one target for a post-Sha backwater retaliation.
    pub fn record_sha_backwater_target(
        &mut self,
        source: PlayerId,
        card: CardId,
        skill: SkillId,
        target: PlayerId,
    ) -> bool {
        let Some(record) = self
            .sha_resolution_records
            .iter_mut()
            .find(|record| record.source == source && record.card == card)
        else {
            return false;
        };
        if !record.targets.contains(&target) {
            return false;
        }
        if !record.backwater_targets.contains(&(skill, target)) {
            record.backwater_targets.push((skill, target));
            record
                .backwater_targets
                .sort_by_key(|(skill, target)| (skill.raw(), target.raw()));
        }
        true
    }

    /// Remember that this exact Sha made one of its targets enter dying.
    pub fn mark_sha_target_entered_dying(
        &mut self,
        source: PlayerId,
        card: CardId,
        target: PlayerId,
    ) {
        if let Some(record) = self
            .sha_resolution_records
            .iter_mut()
            .find(|record| record.source == source && record.card == card)
        {
            if record.targets.contains(&target) && !record.dying_targets.contains(&target) {
                record.dying_targets.push(target);
                record.dying_targets.sort_by_key(|player| player.raw());
            }
        }
    }

    /// Whether one target still belongs to the exact Sha currently resolving.
    pub fn sha_target_resolving(&self, source: PlayerId, card: CardId, target: PlayerId) -> bool {
        self.sha_resolution_records.iter().any(|record| {
            record.source == source && record.card == card && record.targets.contains(&target)
        })
    }

    /// Attach one physical Shan source card to the exact Sha currently resolving.
    pub fn register_sha_response_card(
        &mut self,
        source: PlayerId,
        card: CardId,
        response_card: CardId,
    ) {
        if response_card.is_none() {
            return;
        }
        if let Some(record) = self
            .sha_resolution_records
            .iter_mut()
            .find(|record| record.source == source && record.card == card)
        {
            if !record.response_cards.contains(&response_card) {
                record.response_cards.push(response_card);
            }
        }
    }

    /// Consume one exact Sha record at its post-use boundary.
    pub fn take_sha_resolution(
        &mut self,
        source: PlayerId,
        card: CardId,
    ) -> Option<ShaResolutionRecord> {
        let index = self
            .sha_resolution_records
            .iter()
            .position(|record| record.source == source && record.card == card)?;
        Some(self.sha_resolution_records.remove(index))
    }

    pub(crate) fn saved_sha_target_exclusion_offset(
        &self,
        card: CardId,
        target: PlayerId,
    ) -> Option<usize> {
        match self.resume_kind {
            ContinuationKind::ResumePlayedSha => (self.resume_payload.get(5) == Some(&card.raw())
                && self.resume_payload.get(1) == Some(&target.raw()))
            .then_some(6),
            ContinuationKind::ResumePlayedShaQueue => {
                let len = *self.resume_payload.get(5)? as usize;
                let end = 6usize.checked_add(len)?;
                let exclusion_offset = if self.resume_payload.get(end)
                    == Some(&Self::SAVED_SHA_RESPONSE_FLAGS_MARKER)
                {
                    end.checked_add(1)?.checked_add(len)?
                } else {
                    end
                };
                (self.resume_payload.get(4) == Some(&card.raw())
                    && self.resume_payload.get(6..end)?.contains(&target.raw()))
                .then_some(exclusion_offset)
            }
            _ => None,
        }
    }

    /// Record that one target is excluded from the saved, currently resolving Sha.
    pub fn exclude_saved_sha_target(&mut self, card: CardId, target: PlayerId) -> bool {
        let Some(offset) = self.saved_sha_target_exclusion_offset(card, target) else {
            return false;
        };
        let Some(already_excluded) = self
            .resume_payload
            .get(offset..)
            .map(|exclusions| exclusions.contains(&target.raw()))
        else {
            return false;
        };
        if !already_excluded {
            self.resume_payload.push(target.raw());
        }
        true
    }

    /// Disable one target's response window in the saved, currently resolving Sha.
    pub fn make_saved_sha_target_unrespondable(
        &mut self,
        source: PlayerId,
        card: CardId,
        target: PlayerId,
    ) -> bool {
        match self.resume_kind {
            ContinuationKind::ResumePlayedSha => {
                if self.resume_payload.first() != Some(&source.raw())
                    || self.resume_payload.get(1) != Some(&target.raw())
                    || self.resume_payload.get(5) != Some(&card.raw())
                {
                    return false;
                }
                let Some(respondable) = self.resume_payload.get_mut(4) else {
                    return false;
                };
                *respondable &= !1;
                true
            }
            ContinuationKind::ResumePlayedShaQueue => {
                if self.resume_payload.first() != Some(&source.raw())
                    || self.resume_payload.get(4) != Some(&card.raw())
                {
                    return false;
                }
                let Some(len) = self
                    .resume_payload
                    .get(5)
                    .copied()
                    .map(|value| value as usize)
                else {
                    return false;
                };
                let Some(targets_end) = 6usize.checked_add(len) else {
                    return false;
                };
                let Some(index) = self
                    .resume_payload
                    .get(6..targets_end)
                    .and_then(|targets| targets.iter().position(|value| *value == target.raw()))
                else {
                    return false;
                };
                if self.resume_payload.get(targets_end)
                    != Some(&Self::SAVED_SHA_RESPONSE_FLAGS_MARKER)
                {
                    let flags = std::iter::once(Self::SAVED_SHA_RESPONSE_FLAGS_MARKER)
                        .chain(std::iter::repeat(1).take(len));
                    self.resume_payload.splice(targets_end..targets_end, flags);
                }
                let Some(flag) = self.resume_payload.get_mut(targets_end + 1 + index) else {
                    return false;
                };
                *flag &= !1;
                true
            }
            _ => false,
        }
    }

    /// Clear choice barrier fields (keeps allocators and budget config).
    pub fn clear_choice(&mut self) {
        self.pending_choice_id = 0;
        self.pending_observer = 0;
        self.choice_revision = 0;
        self.continuation_kind = ContinuationKind::None;
        self.continuation_payload.clear();
        self.vm_cont_frames.clear();
        self.vm_cont_module.clear();
        self.vm_cont_pc = 0;
        self.vm_cont_fuel = 0;
        self.vm_cont_ctx = [0; 4];
        self.vm_cont_skill = 0;
        self.vm_cont_trigger_marker = [0; 3];
        self.optional_trigger = None;
    }

    /// Reset segment budgets to configured max.
    pub fn reset_segment_budgets(&mut self) {
        self.effect_budget = self.effect_budget_max;
        self.trigger_budget = self.trigger_budget_max;
    }
}

impl GameState {
    /// 查询缓存依赖版本（ADR 0703 第②部分：缓存边界）。
    ///
    /// 聚合提交版本与结算内单调哨兵（伤害/恢复/选择序号、RNG 调用数）：
    /// 结算内部的这些推进都会使查询缓存键失效，堵住「同一次结算内
    /// 状态已变、revision 未变导致缓存脏读」的边界。注意：不覆盖
    /// 不经过上述哨兵的静默字段写入——宿主在结算外缓存仍须以
    /// 提交边界为清缓存硬边界（见 QueryContext 文档）。
    pub fn query_dep_version(&self) -> u64 {
        let mut v = self.revision;
        v = v.wrapping_mul(0x100000001b3) ^ self.resolution.next_damage_seq as u64;
        v = v.wrapping_mul(0x100000001b3) ^ self.resolution.next_recovery_seq as u64;
        v = v.wrapping_mul(0x100000001b3) ^ self.resolution.next_choice_seq as u64;
        v = v.wrapping_mul(0x100000001b3) ^ self.rng.call_count;
        v
    }

    /// Create an empty pre-start state.
    pub fn empty() -> Self {
        Self {
            revision: 0,
            ruleset: RulesetRef::default(),
            turn: TurnState::default(),
            hand_limit_bonus: std::collections::BTreeMap::new(),
            extra_effect_count: std::collections::BTreeMap::new(),
            players: PlayerStore::new(),
            group_overrides: std::collections::BTreeMap::new(),
            facility_gained_skills: std::collections::BTreeMap::new(),
            deferred_card_exclusions: Vec::new(),
            ambition_alliances: std::collections::BTreeSet::new(),
            face_down_hand_cards: std::collections::BTreeSet::new(),
            debate_pinned_opinions: std::collections::BTreeMap::new(),
            debate_pending_participants: Vec::new(),
            debate_pending_initiator: 0,
            pending_turn_end_debates: std::collections::BTreeMap::new(),
            content_card_tags: std::collections::BTreeMap::new(),
            cards: CardStore::new(),
            zones: ZoneStore::new(),
            skills: SkillRuntimeStore::new(),
            triggers: ActiveTriggerIndex::new(),
            modifiers: ActiveModifierIndex::new(),
            content: std::sync::Arc::new(ContentRegistry::new()),
            damage_history: std::collections::BTreeSet::new(),
            lifetime_dying_players: std::collections::BTreeSet::new(),
            damage_dealt_totals: std::collections::BTreeMap::new(),
            card_action_totals: std::collections::BTreeMap::new(),
            draw_reshuffle_count: 0,
            lifetime_dying_events: 0,
            lifetime_basic_card_uses: 0,
            lifetime_used_type_groups: 0,
            skill_piles: Vec::new(),
            pending_judge_skips: Vec::new(),
            disabled_judge_zones: std::collections::BTreeSet::new(),
            resolution: ResolutionState::default(),
            rng: RuleRngState::default(),
            outcome: MatchOutcome::InProgress,
        }
    }

    /// Current faction after match-time changes.
    pub fn player_group(&self, player: PlayerId) -> Option<&str> {
        self.group_overrides
            .get(&player)
            .map(String::as_str)
            .or_else(|| {
                self.players
                    .get(player)
                    .and_then(|player| self.content.character(player.character_def))
                    .map(|character| character.group.as_str())
            })
            .filter(|group| !group.is_empty())
    }

    /// Whether `player`'s character is a member of `clan` (e.g. "太原王氏").
    /// Clan membership is a static character attribute resolved via the content registry,
    /// mirroring `player_group`. In guozhan the lieutenant general (`character_def2`) is also
    /// consulted when set.
    pub fn player_has_clan(&self, player: PlayerId, clan: &str) -> bool {
        let Some(p) = self.players.get(player) else {
            return false;
        };
        let main = self.content.character(p.character_def);
        let lt = if p.character_def2 != 0 {
            self.content.character(p.character_def2)
        } else {
            None
        };
        main.into_iter()
            .chain(lt)
            .any(|c| c.clans.iter().any(|c| c == clan))
    }

    /// Whether a skill instance is active after definition-level owner constraints.
    pub fn skill_effectively_enabled(&self, skill: &SkillInstance) -> bool {
        let globally_suppressed = self
            .skills
            .get(self.turn.non_locked_skill_suppression)
            .filter(|source| {
                source.enabled
                    && !source.turn_disabled
                    && source.disabled_by_skills.is_empty()
                    && source.owner == self.turn.current
                    && source.owner != skill.owner
                    && self
                        .players
                        .get(skill.owner)
                        .is_some_and(|player| player.alive && !player.dying)
                    && self
                        .content
                        .skill_metadata(skill.def)
                        .is_some_and(|metadata| !metadata.locked)
            })
            .is_some();
        let target_suppressed = self
            .turn
            .non_locked_skill_suppressed_players
            .contains(&skill.owner)
            && self
                .content
                .skill_metadata(skill.def)
                .is_some_and(|metadata| !metadata.locked);
        let hp_one_blocked = self
            .modifiers
            .for_target(crate::query::ModifierTarget::SkillsDisabledWhenHpOne)
            .iter()
            .any(|modifier| {
                modifier.owner == skill.owner
                    && self
                        .content
                        .skill_metadata(skill.def)
                        .is_some_and(|metadata| !metadata.locked)
            })
            && self
                .players
                .get(skill.owner)
                .is_some_and(|player| player.hp == 1);
        skill.enabled
            && !skill.turn_disabled
            && skill.disabled_by_skills.is_empty()
            && !hp_one_blocked
            && !globally_suppressed
            && !target_suppressed
            && self
                .content
                .skill_metadata(skill.def)
                .is_none_or(|metadata| {
                    !metadata.lord_only
                        || self
                            .players
                            .get(skill.owner)
                            .is_some_and(|player| player.identity == Identity::Lord)
                })
            && !self.guozhan_skill_hidden(skill)
    }

    /// ADR 0598 phase C: in guozhan, a skill attached to a face-down (暗置) general is dormant.
    /// Identity-mode skills carry the `GENERAL_SLOT_NONE` sentinel and are never hidden.
    pub(crate) fn guozhan_skill_hidden(&self, skill: &SkillInstance) -> bool {
        if skill.general_slot == GENERAL_SLOT_NONE {
            return false;
        }
        let Some(slot) = usize::try_from(skill.general_slot).ok() else {
            return false;
        };
        self.players
            .get(skill.owner)
            .is_some_and(|player| slot < player.unseen.len() && player.unseen[slot])
    }

    /// Rebuild trigger/modifier indexes from skill store (snapshot restore / recover).
    pub fn rebuild_derived_indexes(&mut self) {
        self.skill_piles
            .sort_by_key(|pile| (pile.owner.raw(), pile.skill_def));
        let skills: Vec<_> = self
            .skills
            .sorted_live()
            .into_iter()
            .map(|s| (s.id, s.def, s.owner, self.skill_effectively_enabled(s)))
            .collect();
        let content = self.content.clone();
        self.triggers.rebuild_from(skills.iter().copied(), &content);
        self.modifiers
            .rebuild_from(skills.iter().copied(), &content);
    }

    /// Whether a choice barrier is open.
    pub fn has_pending_choice(&self) -> bool {
        self.resolution.pending_choice_id != 0
    }

    /// Cards in a player's hand (zone order).
    pub fn hand_cards(&self, player: engine_api::PlayerId) -> Vec<engine_api::CardId> {
        self.zones
            .find(ZoneKind::Hand, player)
            .and_then(|zid| self.zones.get(zid))
            .map(|z| z.cards.clone())
            .unwrap_or_default()
    }

    /// Hand count.
    pub fn hand_count(&self, player: engine_api::PlayerId) -> u32 {
        self.hand_cards(player).len() as u32
    }

    /// Cards legal at hand-card use/response boundaries without affecting hand-count rules.
    pub fn playable_hand_cards(&self, player: engine_api::PlayerId) -> Vec<engine_api::CardId> {
        let mut cards = self.hand_cards(player);
        if crate::query::hand_card_gained_this_turn_only_for(self, player) {
            cards.retain(|card| self.turn.gained_hand_card(player, *card));
        }
        for pile in self
            .skill_piles
            .iter()
            .filter(|pile| pile.owner == player && pile.usable_as_hand)
        {
            if pile.skill_def == POTNIZUN_DEF
                && self.turn.used_playable_skill_pile(player, pile.skill_def)
            {
                continue;
            }
            if let Some(zone) = self.zones.get(pile.zone) {
                cards.extend(zone.cards.iter().copied());
            }
        }
        for lender in self.players.alive_by_seat() {
            if crate::query::pothaoshi_grant(self, lender, player).is_some() {
                cards.extend(self.hand_cards(lender));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        cards.retain(|card| seen.insert(*card));
        cards
    }

    /// Whether one physical card carries a persistent content tag.
    pub fn card_has_content_tag(&self, card: CardId, tag: u32) -> bool {
        self.content_card_tags
            .get(&card)
            .is_some_and(|tags| tags.contains(&tag))
    }

    /// Attach one persistent content tag to a physical card.
    pub fn add_content_card_tag(&mut self, card: CardId, tag: u32) {
        if card.is_some() && tag != 0 && self.cards.get(card).is_some() {
            self.content_card_tags.entry(card).or_default().insert(tag);
        }
    }

    /// Remove one content tag after its card-scoped contract is consumed.
    pub fn remove_content_card_tag(&mut self, card: CardId, tag: u32) {
        let empty = self.content_card_tags.get_mut(&card).is_some_and(|tags| {
            tags.remove(&tag);
            tags.is_empty()
        });
        if empty {
            self.content_card_tags.remove(&card);
        }
    }

    /// Public expansion cards stored for one player's skill definition.
    pub fn skill_pile_cards(
        &self,
        player: engine_api::PlayerId,
        skill_def: u32,
    ) -> Vec<engine_api::CardId> {
        self.skill_piles
            .iter()
            .find(|pile| pile.owner == player && pile.skill_def == skill_def)
            .and_then(|pile| self.zones.get(pile.zone))
            .map(|zone| zone.cards.clone())
            .unwrap_or_default()
    }

    /// Face-down expansion cards in one skill-owned pile.
    pub fn hidden_skill_pile_cards(
        &self,
        player: engine_api::PlayerId,
        skill_def: u32,
    ) -> Vec<engine_api::CardId> {
        self.skill_pile_cards(player, skill_def)
            .into_iter()
            .filter(|card| self.cards.get(*card).is_some_and(|card| !card.face_up))
            .collect()
    }

    /// Face-up expansion cards in one skill-owned pile.
    pub fn revealed_skill_pile_cards(
        &self,
        player: engine_api::PlayerId,
        skill_def: u32,
    ) -> Vec<engine_api::CardId> {
        self.skill_pile_cards(player, skill_def)
            .into_iter()
            .filter(|card| self.cards.get(*card).is_some_and(|card| card.face_up))
            .collect()
    }
}
