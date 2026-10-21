//! Runtime skill state for `twxiaolian_redirect_state_tag` and related helpers.

use super::*;

impl SkillInstance {
    pub(crate) const TWXIAOLIAN_REDIRECT_STATE_TAG: [u8; 4] = *b"TXL1";

    /// The recorded card ignores armor while resolving its effects.
    pub(crate) const CARD_USE_POLICY_IGNORE_ARMOR: u8 = 1 << 0;

    /// The recorded card cannot be answered by Shan or Wuxie.
    pub(crate) const CARD_USE_POLICY_UNRESPONDABLE: u8 = 1 << 1;

    /// The owner draws after each damage event caused by the recorded card.
    pub(crate) const CARD_USE_POLICY_DRAW_ON_DAMAGE: u8 = 1 << 2;

    /// The recorded card ignores ordinary target-distance limits.
    pub(crate) const CARD_USE_POLICY_DISTANCE_UNLIMITED: u8 = 1 << 3;

    pub(crate) fn set_twlihuo_dying_cost(&mut self, card: engine_api::CardId) {
        self.state_blob = Self::TWLIHUO_DYING_COST_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        self.state_blob.push(0);
    }

    pub(crate) fn twlihuo_dying_cost_card(&self) -> Option<engine_api::CardId> {
        (self.state_blob.len() == 9
            && self.state_blob.get(..4) == Some(Self::TWLIHUO_DYING_COST_STATE_TAG.as_slice()))
        .then(|| {
            engine_api::CardId::new(u32::from_le_bytes(
                self.state_blob[4..8].try_into().unwrap(),
            ))
        })
        .filter(|card| card.is_some())
    }

    pub(crate) fn mark_twlihuo_caused_dying(&mut self) {
        if self.twlihuo_dying_cost_card().is_some() {
            self.state_blob[8] = 1;
        }
    }

    pub(crate) fn take_twlihuo_dying_cost(&mut self, card: engine_api::CardId) -> Option<bool> {
        (self.twlihuo_dying_cost_card() == Some(card)).then(|| {
            let caused_dying = self.state_blob[8] != 0;
            self.state_blob.clear();
            caused_dying
        })
    }

    pub(crate) fn set_twxiaolian_redirect(
        &mut self,
        card: engine_api::CardId,
        original_target: PlayerId,
    ) {
        self.state_blob = Self::TWXIAOLIAN_REDIRECT_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        self.state_blob
            .extend_from_slice(&original_target.raw().to_le_bytes());
    }

    pub(crate) fn twxiaolian_redirect(&self) -> Option<(engine_api::CardId, PlayerId)> {
        (self.state_blob.len() == 12
            && self.state_blob.get(..4) == Some(Self::TWXIAOLIAN_REDIRECT_STATE_TAG.as_slice()))
        .then(|| {
            (
                engine_api::CardId::new(u32::from_le_bytes(
                    self.state_blob[4..8].try_into().unwrap(),
                )),
                PlayerId::new(u32::from_le_bytes(
                    self.state_blob[8..12].try_into().unwrap(),
                )),
            )
        })
        .filter(|(card, target)| card.is_some() && target.is_some())
    }

    pub(crate) fn clear_twxiaolian_redirect(&mut self) {
        if self.twxiaolian_redirect().is_some() {
            self.state_blob.clear();
        }
    }

    /// Whether Longnu is currently on its red-card/fire-Sha branch.
    pub(crate) fn nzry_longnu_fire_branch(&self) -> bool {
        self.state_blob.len() == 5
            && self.state_blob.get(..4) == Some(Self::NZRY_LONGNU_STATE_TAG.as_slice())
            && self.state_blob[4] != 0
    }

    /// Persist Longnu's current conversion branch.
    pub(crate) fn set_nzry_longnu_fire_branch(&mut self, fire: bool) {
        self.state_blob.clear();
        self.state_blob
            .extend_from_slice(&Self::NZRY_LONGNU_STATE_TAG);
        self.state_blob.push(u8::from(fire));
    }

    /// Target-side helper currently owned by an active Duorui borrow.
    pub(crate) fn drlt_duorui_helper(&self) -> Option<SkillId> {
        (self.state_blob.len() == 8
            && self.state_blob.get(..4) == Some(Self::DRLT_DUORUI_MAIN_STATE_TAG.as_slice()))
        .then(|| {
            SkillId::new(u32::from_le_bytes(
                self.state_blob[4..8].try_into().unwrap(),
            ))
        })
        .filter(|skill| skill.is_some())
    }

    /// Mark this Duorui instance as occupied by one target-side helper.
    pub(crate) fn set_drlt_duorui_helper(&mut self, helper: SkillId) {
        self.state_blob.clear();
        self.state_blob
            .extend_from_slice(&Self::DRLT_DUORUI_MAIN_STATE_TAG);
        self.state_blob
            .extend_from_slice(&helper.raw().to_le_bytes());
    }

    /// Borrower, Duorui instance, disabled target skill, and temporary borrowed skill.
    pub(crate) fn drlt_duorui_record(&self) -> Option<(PlayerId, SkillId, SkillId, SkillId)> {
        if self.state_blob.len() != 20
            || self.state_blob.get(..4) != Some(Self::DRLT_DUORUI_HELPER_STATE_TAG.as_slice())
        {
            return None;
        }
        let raw = |start| {
            u32::from_le_bytes(
                self.state_blob[start..start + 4]
                    .try_into()
                    .expect("fixed Duorui state"),
            )
        };
        Some((
            PlayerId::new(raw(4)),
            SkillId::new(raw(8)),
            SkillId::new(raw(12)),
            SkillId::new(raw(16)),
        ))
    }

    /// Store one target-side Duorui cleanup record.
    pub(crate) fn set_drlt_duorui_record(
        &mut self,
        borrower: PlayerId,
        duorui: SkillId,
        target_skill: SkillId,
        borrowed_skill: SkillId,
    ) {
        self.state_blob.clear();
        self.state_blob
            .extend_from_slice(&Self::DRLT_DUORUI_HELPER_STATE_TAG);
        for raw in [
            borrower.raw(),
            duorui.raw(),
            target_skill.raw(),
            borrowed_skill.raw(),
        ] {
            self.state_blob.extend_from_slice(&raw.to_le_bytes());
        }
    }

    /// Command revision at which a failed proactive delegated-card request may not be retried.
    pub(crate) fn delegated_retry_lock_revision(&self) -> Option<u64> {
        (self.state_blob.len() == 12
            && self.state_blob.get(..4) == Some(Self::DELEGATED_RETRY_LOCK_STATE_TAG.as_slice()))
        .then(|| u64::from_le_bytes(self.state_blob[4..12].try_into().unwrap()))
    }

    pub(crate) fn set_delegated_retry_lock_revision(&mut self, revision: u64) {
        self.state_blob.clear();
        self.state_blob
            .extend_from_slice(&Self::DELEGATED_RETRY_LOCK_STATE_TAG);
        self.state_blob.extend_from_slice(&revision.to_le_bytes());
    }

    /// Decode turn-scoped active-skill category and target uniqueness state.
    pub(crate) fn active_unique_state(&self) -> Option<(u8, Vec<PlayerId>)> {
        if self
            .state_blob
            .get(..4)
            .is_some_and(|tag| tag == Self::ACTIVE_COMPARE_HISTORY_STATE_TAG)
        {
            let (mut targets, _) = self.active_compare_history()?;
            targets.sort_by_key(|player| player.raw());
            return Some((0, targets));
        }
        if self.state_blob.len() < 5
            || (self.state_blob.len() - 5) % 4 != 0
            || self.state_blob.get(..4) != Some(Self::ACTIVE_UNIQUE_STATE_TAG.as_slice())
        {
            return None;
        }
        let category_mask = self.state_blob[4] & 0b111;
        let mut targets = Vec::new();
        for chunk in self.state_blob[5..].chunks_exact(4) {
            let target = PlayerId::new(u32::from_le_bytes(chunk.try_into().ok()?));
            if target.is_none()
                || targets
                    .last()
                    .is_some_and(|previous: &PlayerId| previous.raw() >= target.raw())
            {
                return None;
            }
            targets.push(target);
        }
        Some((category_mask, targets))
    }

    /// Atomically record one logical category bit and one target for a turn-scoped active skill.
    pub(crate) fn record_active_unique_use(&mut self, category_bit: u8, target: PlayerId) -> bool {
        if category_bit == 0 || category_bit & !0b111 != 0 || target.is_none() {
            return false;
        }
        let (mut mask, mut targets) = if self.state_blob.is_empty() {
            (0, Vec::new())
        } else if let Some(state) = self.active_unique_state() {
            state
        } else {
            return false;
        };
        if mask & category_bit != 0 || targets.contains(&target) {
            return false;
        }
        mask |= category_bit;
        targets.push(target);
        targets.sort_by_key(|player| player.raw());
        let mut state = Self::ACTIVE_UNIQUE_STATE_TAG.to_vec();
        state.push(mask);
        for target in targets {
            state.extend_from_slice(&target.raw().to_le_bytes());
        }
        self.state_blob = state;
        true
    }

    /// Record one target for a target-only turn-scoped active-skill uniqueness contract.
    pub(crate) fn record_active_target(&mut self, target: PlayerId) -> bool {
        if target.is_none() {
            return false;
        }
        let (mask, mut targets) = if self.state_blob.is_empty() {
            (0, Vec::new())
        } else if self.state_blob.get(..4) == Some(Self::ACTIVE_UNIQUE_STATE_TAG.as_slice()) {
            self.active_unique_state().unwrap_or_default()
        } else {
            return false;
        };
        if targets.contains(&target) {
            return false;
        }
        targets.push(target);
        targets.sort_by_key(|player| player.raw());
        let mut state = Self::ACTIVE_UNIQUE_STATE_TAG.to_vec();
        state.push(mask);
        for target in targets {
            state.extend_from_slice(&target.raw().to_le_bytes());
        }
        self.state_blob = state;
        true
    }

    /// Decode target uniqueness state scoped to the current Play phase.
    pub(crate) fn active_play_phase_targets(&self) -> Option<Vec<PlayerId>> {
        if self.state_blob.get(..4) == Some(Self::TRACKED_PLAY_TARGETS_STATE_TAG.as_slice()) {
            let count = u32::from_le_bytes(self.state_blob.get(4..8)?.try_into().ok()?) as usize;
            let end = 8usize.checked_add(count.checked_mul(4)?)?;
            return self.state_blob.get(8..end)?.chunks_exact(4).try_fold(
                Vec::new(),
                |mut targets, chunk| {
                    let target = PlayerId::new(u32::from_le_bytes(chunk.try_into().ok()?));
                    if target.is_none()
                        || targets
                            .last()
                            .is_some_and(|previous: &PlayerId| previous.raw() >= target.raw())
                    {
                        return None;
                    }
                    targets.push(target);
                    Some(targets)
                },
            );
        }
        if self.state_blob.len() < 4
            || (self.state_blob.len() - 4) % 4 != 0
            || self.state_blob.get(..4)
                != Some(Self::ACTIVE_PLAY_PHASE_TARGETS_STATE_TAG.as_slice())
        {
            return None;
        }
        let mut targets = Vec::new();
        for chunk in self.state_blob[4..].chunks_exact(4) {
            let target = PlayerId::new(u32::from_le_bytes(chunk.try_into().ok()?));
            if target.is_none()
                || targets
                    .last()
                    .is_some_and(|previous: &PlayerId| previous.raw() >= target.raw())
            {
                return None;
            }
            targets.push(target);
        }
        Some(targets)
    }

    /// Record one target for a target-only Play-phase uniqueness contract.
    pub(crate) fn record_active_play_phase_target(&mut self, target: PlayerId) -> bool {
        if target.is_none() {
            return false;
        }
        let tracked = self.tracked_cards();
        let mut targets = if self.state_blob.is_empty() {
            Vec::new()
        } else if self.state_blob.get(..4)
            == Some(Self::ACTIVE_PLAY_PHASE_TARGETS_STATE_TAG.as_slice())
            || self.state_blob.get(..4) == Some(Self::TRACKED_PLAY_TARGETS_STATE_TAG.as_slice())
        {
            self.active_play_phase_targets().unwrap_or_default()
        } else {
            return false;
        };
        if targets.contains(&target) {
            return false;
        }
        targets.push(target);
        targets.sort_by_key(|player| player.raw());
        if tracked.is_empty() {
            self.state_blob = Self::ACTIVE_PLAY_PHASE_TARGETS_STATE_TAG.to_vec();
            for target in targets {
                self.state_blob
                    .extend_from_slice(&target.raw().to_le_bytes());
            }
        } else {
            self.state_blob = Self::TRACKED_PLAY_TARGETS_STATE_TAG.to_vec();
            self.state_blob
                .extend_from_slice(&(targets.len() as u32).to_le_bytes());
            for target in targets {
                self.state_blob
                    .extend_from_slice(&target.raw().to_le_bytes());
            }
            for card in tracked {
                self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
            }
        }
        true
    }

    /// Record one comparison target and the strict non-win subset in activation order.
    pub(crate) fn record_active_compare_target(&mut self, target: PlayerId) -> bool {
        if target.is_none() {
            return false;
        }
        let (mut targets, failures) = if self.state_blob.is_empty() {
            (Vec::new(), Vec::new())
        } else if let Some(history) = self.active_compare_history() {
            history
        } else {
            return false;
        };
        if targets.contains(&target) {
            return false;
        }
        targets.push(target);
        self.write_active_compare_history(&targets, &failures)
    }

    /// Record one already-used comparison target as a strict non-win.
    pub(crate) fn record_active_compare_failure(&mut self, target: PlayerId) -> Option<usize> {
        let (targets, mut failures) = self.active_compare_history()?;
        if !targets.contains(&target) || failures.contains(&target) {
            return None;
        }
        failures.push(target);
        self.write_active_compare_history(&targets, &failures)
            .then_some(failures.len())
    }

    /// Failed comparison targets in activation order.
    pub(crate) fn active_compare_failures(&self) -> Option<Vec<PlayerId>> {
        self.active_compare_history().map(|(_, failures)| failures)
    }

    pub(crate) fn active_compare_history(&self) -> Option<(Vec<PlayerId>, Vec<PlayerId>)> {
        if self.state_blob.len() < 6
            || self.state_blob.get(..4) != Some(Self::ACTIVE_COMPARE_HISTORY_STATE_TAG.as_slice())
        {
            return None;
        }
        let target_count = usize::from(self.state_blob[4]);
        let failure_count = usize::from(self.state_blob[5]);
        if self.state_blob.len() != 6 + (target_count + failure_count) * 4 {
            return None;
        }
        let mut values = self.state_blob[6..]
            .chunks_exact(4)
            .map(|chunk| {
                let bytes: [u8; 4] = chunk.try_into().ok()?;
                Some(PlayerId::new(u32::from_le_bytes(bytes)))
            })
            .collect::<Option<Vec<_>>>()?;
        if values.iter().any(|player| player.is_none()) {
            return None;
        }
        let failures = values.split_off(target_count);
        if values
            .iter()
            .enumerate()
            .any(|(index, target)| values[..index].contains(target))
            || failures.iter().enumerate().any(|(index, target)| {
                !values.contains(target) || failures[..index].contains(target)
            })
        {
            return None;
        }
        Some((values, failures))
    }

    pub(crate) fn write_active_compare_history(
        &mut self,
        targets: &[PlayerId],
        failures: &[PlayerId],
    ) -> bool {
        let Ok(target_count) = u8::try_from(targets.len()) else {
            return false;
        };
        let Ok(failure_count) = u8::try_from(failures.len()) else {
            return false;
        };
        let mut state = Self::ACTIVE_COMPARE_HISTORY_STATE_TAG.to_vec();
        state.extend([target_count, failure_count]);
        for player in targets.iter().chain(failures) {
            state.extend_from_slice(&player.raw().to_le_bytes());
        }
        self.state_blob = state;
        true
    }

    /// Clear only the generic turn-scoped active uniqueness shape, preserving unrelated state.
    pub(crate) fn clear_active_unique_state(&mut self) {
        if self.state_blob.get(..4).is_some_and(|tag| {
            tag == Self::ACTIVE_UNIQUE_STATE_TAG || tag == Self::ACTIVE_COMPARE_HISTORY_STATE_TAG
        }) {
            self.state_blob.clear();
        }
    }

    /// Store one logical basic/trick/equipment category for the current Play phase.
    pub(crate) fn set_remembered_category(&mut self, category_bit: u8) -> bool {
        if category_bit == 0 || category_bit & !0b111 != 0 {
            return false;
        }
        self.state_blob = Self::REMEMBERED_CATEGORY_STATE_TAG.to_vec();
        self.state_blob.push(category_bit);
        true
    }

    /// Decode the remembered logical card category, if this instance uses that state shape.
    pub(crate) fn remembered_category(&self) -> Option<u8> {
        if self.state_blob.len() != 5
            || self.state_blob.get(..4) != Some(Self::REMEMBERED_CATEGORY_STATE_TAG.as_slice())
        {
            return None;
        }
        let bit = self.state_blob[4];
        (bit != 0 && bit & !0b111 == 0).then_some(bit)
    }

    /// Clear only the Play-phase remembered-category state shape.
    pub(crate) fn clear_remembered_category(&mut self) {
        if self
            .state_blob
            .get(..4)
            .is_some_and(|tag| tag == Self::REMEMBERED_CATEGORY_STATE_TAG)
        {
            self.state_blob.clear();
        }
    }

    /// Store one Play-phase player and one logical basic/trick/equipment category.
    pub(crate) fn set_bound_category_target(&mut self, target: PlayerId, category_bit: u8) -> bool {
        if target.is_none() || category_bit == 0 || category_bit & !0b111 != 0 {
            return false;
        }
        self.state_blob = Self::BOUND_CATEGORY_TARGET_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
        self.state_blob.push(category_bit);
        true
    }

    /// Decode the Play-phase player and logical category bound to this skill instance.
    pub(crate) fn bound_category_target(&self) -> Option<(PlayerId, u8)> {
        if self.state_blob.len() != 9
            || self.state_blob.get(..4) != Some(Self::BOUND_CATEGORY_TARGET_STATE_TAG.as_slice())
        {
            return None;
        }
        let target = PlayerId::new(u32::from_le_bytes(self.state_blob[4..8].try_into().ok()?));
        let category_bit = self.state_blob[8];
        (target.is_some() && category_bit != 0 && category_bit & !0b111 == 0)
            .then_some((target, category_bit))
    }

    /// Clear only the Play-phase player/category binding state shape.
    pub(crate) fn clear_bound_category_target(&mut self) {
        if self
            .state_blob
            .get(..4)
            .is_some_and(|tag| tag == Self::BOUND_CATEGORY_TARGET_STATE_TAG)
        {
            self.state_blob.clear();
        }
    }

    /// Record the other player whose current Play phase owns a delayed gift consequence.
    pub(crate) fn set_phase_gift_target(&mut self, target: PlayerId) -> bool {
        if target.is_none() {
            return false;
        }
        self.state_blob = Self::PHASE_GIFT_TARGET_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
        true
    }

    /// Decode the current delayed gift target.
    pub(crate) fn phase_gift_target(&self) -> Option<PlayerId> {
        if self.state_blob.len() != 8
            || self.state_blob.get(..4) != Some(Self::PHASE_GIFT_TARGET_STATE_TAG.as_slice())
        {
            return None;
        }
        let target = PlayerId::new(u32::from_le_bytes(self.state_blob[4..8].try_into().ok()?));
        target.is_some().then_some(target)
    }

    /// Clear only the delayed Play-phase gift target state shape.
    pub(crate) fn clear_phase_gift_target(&mut self) {
        if self
            .state_blob
            .get(..4)
            .is_some_and(|tag| tag == Self::PHASE_GIFT_TARGET_STATE_TAG)
        {
            self.state_blob.clear();
        }
    }

    /// Encode the shared, versioned state shape used by target-bound helper skills.
    pub(crate) fn bound_target_state(target: PlayerId) -> Vec<u8> {
        let mut state = Self::BOUND_TARGET_STATE_TAG.to_vec();
        state.extend_from_slice(&target.raw().to_le_bytes());
        state
    }

    /// Decode this instance's shared target-bound state, if present and well formed.
    pub(crate) fn bound_target(&self) -> Option<PlayerId> {
        if self.state_blob.len() != 8
            || self.state_blob.get(..4) != Some(Self::BOUND_TARGET_STATE_TAG.as_slice())
        {
            return None;
        }
        let raw = u32::from_le_bytes(self.state_blob[4..8].try_into().ok()?);
        let target = PlayerId::new(raw);
        target.is_some().then_some(target)
    }

    /// Canonical source set for helpers that expire at each source's next turn begin.
    pub(crate) fn source_turn_bounds(&self) -> Option<Vec<PlayerId>> {
        if self.state_blob.len() < 8
            || (self.state_blob.len() - 4) % 4 != 0
            || self.state_blob.get(..4) != Some(Self::SOURCE_TURN_BOUND_STATE_TAG.as_slice())
        {
            return None;
        }
        let mut sources = Vec::new();
        for chunk in self.state_blob[4..].chunks_exact(4) {
            let source = PlayerId::new(u32::from_le_bytes(chunk.try_into().ok()?));
            if source.is_none() {
                return None;
            }
            sources.push(source);
        }
        sources.sort_by_key(|source| source.raw());
        sources.dedup();
        (!sources.is_empty()).then_some(sources)
    }

    /// Add one source to a source-turn-bound helper.
    pub(crate) fn add_source_turn_bound(&mut self, source: PlayerId) -> bool {
        if source.is_none() {
            return false;
        }
        let mut sources = self.source_turn_bounds().unwrap_or_default();
        if sources.contains(&source) {
            return true;
        }
        sources.push(source);
        sources.sort_by_key(|source| source.raw());
        self.state_blob = Self::SOURCE_TURN_BOUND_STATE_TAG.to_vec();
        for source in sources {
            self.state_blob
                .extend_from_slice(&source.raw().to_le_bytes());
        }
        true
    }

    /// Remove one expired source; returns whether another source remains.
    pub(crate) fn remove_source_turn_bound(&mut self, source: PlayerId) -> Option<bool> {
        let mut sources = self.source_turn_bounds()?;
        sources.retain(|candidate| *candidate != source);
        if sources.is_empty() {
            self.state_blob.clear();
            return Some(false);
        }
        self.state_blob = Self::SOURCE_TURN_BOUND_STATE_TAG.to_vec();
        for source in sources {
            self.state_blob
                .extend_from_slice(&source.raw().to_le_bytes());
        }
        Some(true)
    }

    /// Encode the source player and red/black mask for a temporary card-color lock.
    pub(crate) fn card_color_lock_state(source: PlayerId, color_mask: u8) -> Vec<u8> {
        let mut state = Self::CARD_COLOR_LOCK_STATE_TAG.to_vec();
        state.extend_from_slice(&source.raw().to_le_bytes());
        state.push(color_mask & 0b11);
        state
    }

    /// Decode a card-color lock's source and non-empty red/black mask.
    pub(crate) fn card_color_lock(&self) -> Option<(PlayerId, u8)> {
        if self.state_blob.len() != 9
            || self.state_blob.get(..4) != Some(Self::CARD_COLOR_LOCK_STATE_TAG.as_slice())
        {
            return None;
        }
        let raw = u32::from_le_bytes(self.state_blob[4..8].try_into().ok()?);
        let source = PlayerId::new(raw);
        let color_mask = self.state_blob[8] & 0b11;
        (source.is_some() && color_mask != 0).then_some((source, color_mask))
    }

    /// Encode a canonical exact-card policy table for temporary helper skills.
    pub(crate) fn card_use_policy_state(entries: &[(engine_api::CardId, u8)]) -> Vec<u8> {
        let mut canonical = std::collections::BTreeMap::new();
        for (card, mask) in entries {
            let mask = mask & 0b111;
            if card.is_some() && mask != 0 {
                canonical
                    .entry(card.raw())
                    .and_modify(|current| *current |= mask)
                    .or_insert(mask);
            }
        }
        let mut state = Self::CARD_USE_POLICY_STATE_TAG.to_vec();
        for (card, mask) in canonical {
            state.extend_from_slice(&card.to_le_bytes());
            state.push(mask);
        }
        state
    }

    /// Decode the exact-card policy mask for `card`, or zero for another state shape.
    /// Decode the exact-card policy mask for `card`, or zero for another state shape.
    pub fn card_use_policy(&self, card: engine_api::CardId) -> u8 {
        if let Some((policy_card, mask, _)) = self.phase_card_use_policy() {
            return if policy_card == card { mask } else { 0 };
        }
        if card.is_none()
            || self.state_blob.len() < 9
            || (self.state_blob.len() - 4) % 5 != 0
            || self.state_blob.get(..4) != Some(Self::CARD_USE_POLICY_STATE_TAG.as_slice())
        {
            return 0;
        }
        self.state_blob[4..]
            .chunks_exact(5)
            .find_map(|entry| {
                let raw = u32::from_le_bytes(entry[..4].try_into().ok()?);
                (raw == card.raw()).then_some(entry[4] & 0b111)
            })
            .unwrap_or(0)
    }

    /// Encode one exact-card policy together with the owner's damage-total baseline.
    pub(crate) fn phase_card_use_policy_state(
        card: engine_api::CardId,
        policy_mask: u8,
        damage_baseline: u32,
    ) -> Vec<u8> {
        let mut state = Self::PHASE_CARD_USE_POLICY_STATE_TAG.to_vec();
        state.extend_from_slice(&card.raw().to_le_bytes());
        state.push(policy_mask & 0b1111);
        state.extend_from_slice(&damage_baseline.to_le_bytes());
        state
    }

    /// Decode a phase-scoped exact-card policy and its damage-total baseline.
    pub(crate) fn phase_card_use_policy(&self) -> Option<(engine_api::CardId, u8, u32)> {
        if self.state_blob.len() != 13
            || self.state_blob.get(..4) != Some(Self::PHASE_CARD_USE_POLICY_STATE_TAG.as_slice())
        {
            return None;
        }
        let card =
            engine_api::CardId::new(u32::from_le_bytes(self.state_blob[4..8].try_into().ok()?));
        let mask = self.state_blob[8] & 0b1111;
        let damage_baseline = u32::from_le_bytes(self.state_blob[9..13].try_into().ok()?);
        (card.is_some() && mask != 0).then_some((card, mask, damage_baseline))
    }

    /// Clear only the phase-scoped exact-card policy state shape.
    pub(crate) fn clear_phase_card_use_policy_state(&mut self) {
        if self
            .state_blob
            .get(..4)
            .is_some_and(|tag| tag == Self::PHASE_CARD_USE_POLICY_STATE_TAG)
        {
            self.state_blob.clear();
        }
    }

    /// Decode all exact-card policy entries in canonical card-id order.
    pub(crate) fn card_use_policies(&self) -> Option<Vec<(engine_api::CardId, u8)>> {
        if self.state_blob.len() < 9
            || (self.state_blob.len() - 4) % 5 != 0
            || self.state_blob.get(..4) != Some(Self::CARD_USE_POLICY_STATE_TAG.as_slice())
        {
            return None;
        }
        let entries: Vec<_> = self.state_blob[4..]
            .chunks_exact(5)
            .filter_map(|entry| {
                let card = engine_api::CardId::new(u32::from_le_bytes(entry[..4].try_into().ok()?));
                let mask = entry[4] & 0b111;
                (card.is_some() && mask != 0).then_some((card, mask))
            })
            .collect();
        (!entries.is_empty()).then_some(entries)
    }

    /// Decode exact cards remembered while they continuously remain in the owner's hand.
    pub(crate) fn remembered_hand_cards(&self) -> Option<Vec<CardId>> {
        if self.state_blob.is_empty() {
            return Some(Vec::new());
        }
        if self.state_blob.len() < 8
            || (self.state_blob.len() - 4) % 4 != 0
            || self.state_blob.get(..4) != Some(Self::REMEMBERED_HAND_CARDS_STATE_TAG.as_slice())
        {
            return None;
        }
        let cards: Vec<_> = self.state_blob[4..]
            .chunks_exact(4)
            .map(|chunk| CardId::new(u32::from_le_bytes(chunk.try_into().unwrap())))
            .collect();
        cards
            .iter()
            .all(|card| card.is_some())
            .then_some(cards)
            .filter(|cards| cards.windows(2).all(|pair| pair[0].raw() < pair[1].raw()))
    }

    /// Merge exact newly drawn cards into the remembered set.
    pub(crate) fn remember_hand_cards(&mut self, cards: &[CardId]) -> bool {
        let Some(mut remembered) = self.remembered_hand_cards() else {
            return false;
        };
        remembered.extend(cards.iter().copied().filter(|card| card.is_some()));
        remembered.sort_by_key(|card| card.raw());
        remembered.dedup();
        self.set_remembered_hand_cards(&remembered);
        true
    }

    /// Take and clear the remembered set.
    pub(crate) fn take_remembered_hand_cards(&mut self) -> Option<Vec<CardId>> {
        let cards = self.remembered_hand_cards()?;
        self.state_blob.clear();
        Some(cards)
    }

    /// Forget one remembered card when it leaves the owner's hand.
    pub(crate) fn forget_remembered_hand_card(&mut self, card: CardId) {
        if self.state_blob.get(..4) != Some(Self::REMEMBERED_HAND_CARDS_STATE_TAG.as_slice()) {
            return;
        }
        let Some(mut cards) = self.remembered_hand_cards() else {
            self.state_blob.clear();
            return;
        };
        cards.retain(|remembered| *remembered != card);
        self.set_remembered_hand_cards(&cards);
    }

    pub(crate) fn set_remembered_hand_cards(&mut self, cards: &[CardId]) {
        self.state_blob.clear();
        if cards.is_empty() {
            return;
        }
        self.state_blob
            .extend_from_slice(&Self::REMEMBERED_HAND_CARDS_STATE_TAG);
        for card in cards {
            self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        }
    }

    pub(crate) fn has_lifetime_target(&self, target: PlayerId) -> bool {
        self.state_blob.get(..4) == Some(Self::LIFETIME_TARGETS_STATE_TAG.as_slice())
            && self.state_blob[4..]
                .chunks_exact(4)
                .any(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()) == target.raw())
    }

    pub(crate) fn record_lifetime_target(&mut self, target: PlayerId) -> bool {
        if target.is_none() || self.has_lifetime_target(target) {
            return false;
        }
        let mut targets = if self.state_blob.is_empty() {
            Vec::new()
        } else if self.state_blob.get(..4) == Some(Self::LIFETIME_TARGETS_STATE_TAG.as_slice())
            && (self.state_blob.len() - 4) % 4 == 0
        {
            self.state_blob[4..]
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect()
        } else {
            return false;
        };
        targets.push(target.raw());
        targets.sort_unstable();
        self.state_blob = Self::LIFETIME_TARGETS_STATE_TAG.to_vec();
        for target in targets {
            self.state_blob.extend_from_slice(&target.to_le_bytes());
        }
        true
    }
}

impl SkillRuntimeStore {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate.
    pub fn alloc(&mut self, mut skill: SkillInstance) -> SkillId {
        self.next_id = self.next_id.max(1);
        let id = SkillId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        skill.id = id;
        let idx = (id.raw() - 1) as usize;
        while self.skills.len() <= idx {
            self.skills.push(None);
        }
        self.skills[idx] = Some(skill);
        id
    }

    /// Get.
    pub fn get(&self, id: SkillId) -> Option<&SkillInstance> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.skills.get(idx).and_then(|s| s.as_ref())
    }

    /// Get mut.
    pub fn get_mut(&mut self, id: SkillId) -> Option<&mut SkillInstance> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.skills.get_mut(idx).and_then(|s| s.as_mut())
    }

    /// Remove instance (leaves hole; id not reused).
    pub fn remove(&mut self, id: SkillId) -> Option<SkillInstance> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.skills.get_mut(idx).and_then(|s| s.take())
    }

    /// Sorted live.
    pub fn sorted_live(&self) -> Vec<&SkillInstance> {
        let mut v: Vec<&SkillInstance> = self.skills.iter().filter_map(|s| s.as_ref()).collect();
        v.sort_by_key(|s| s.id.raw());
        v
    }

    /// Remove one disabling skill id from every live instance.
    pub fn remove_disable_reason(&mut self, reason: SkillId) -> bool {
        let mut changed = false;
        for skill in self.skills.iter_mut().filter_map(Option::as_mut) {
            let before = skill.disabled_by_skills.len();
            skill.disabled_by_skills.retain(|found| *found != reason);
            changed |= skill.disabled_by_skills.len() != before;
        }
        changed
    }

    /// Clear trigger counters scoped to one play phase.
    pub fn reset_play_phase_fire_counts(&mut self) {
        for skill in self.skills.iter_mut().filter_map(Option::as_mut) {
            skill.play_phase_fire_count = 0;
        }
    }

    /// Reset one skill instance's per-turn / per-phase usage and trigger counters, mirroring
    /// `reset_turn_state` (command/turn.rs) but scoped to a single `SkillId`. This is the
    /// authoritative implementation of upstream `player.refreshSkill()` for the clan "中流"
    /// family: it lets a locked skill fire/activate again within the same turn/phase.
    ///
    /// Content-owned persistent state (`lifetime_fire_count`, `mark`, `content_flags`,
    /// `view_as_*`) is intentionally left untouched, matching the engine's natural lifecycle
    /// where those fields are never cleared by turn/phase rollover.
    pub fn refresh_skill(&mut self, skill: SkillId) -> bool {
        let Some(s) = self.get_mut(skill) else {
            return false;
        };
        s.use_count = 0;
        s.turn_fire_count = 0;
        s.play_phase_fire_count = 0;
        s.play_phase_use_count = 0;
        s.play_phase_category_mask = 0;
        s.active_turn_banned = false;
        s.turn_disabled = false;
        s.round_fire_round = 0;
        s.clear_active_unique_state();
        s.clear_xinyaoming_turn_state();
        s.clear_twguoyi_turn_state();
        s.clear_remembered_category();
        s.clear_bound_category_target();
        true
    }

    /// Generic per-skill keyed content storage: read the value for `key`, or 0 if absent.
    pub fn storage_get(&self, skill: SkillId, key: u32) -> u32 {
        self.get(skill)
            .and_then(|s| s.content_storage.get(&key).copied())
            .unwrap_or(0)
    }

    /// Generic per-skill keyed content storage: write `value` under `key`. Returns false if the
    /// skill instance does not exist.
    pub fn storage_set(&mut self, skill: SkillId, key: u32, value: u32) -> bool {
        if let Some(s) = self.get_mut(skill) {
            s.content_storage.insert(key, value);
            true
        } else {
            false
        }
    }

    /// Generic per-skill keyed content storage: whether `key` is present.
    pub fn storage_has(&self, skill: SkillId, key: u32) -> bool {
        self.get(skill)
            .is_some_and(|s| s.content_storage.contains_key(&key))
    }

    /// Generic per-skill keyed content storage: remove `key`. Returns whether it was present.
    pub fn storage_remove(&mut self, skill: SkillId, key: u32) -> bool {
        self.get_mut(skill)
            .is_some_and(|s| s.content_storage.remove(&key).is_some())
    }

    /// Clear active-skill invocation counts when a new play phase begins.
    pub fn reset_play_phase_use_counts(&mut self) {
        for skill in self.skills.iter_mut().filter_map(Option::as_mut) {
            skill.play_phase_use_count = 0;
            skill.play_phase_category_mask = 0;
            if skill.state_blob.get(..4)
                == Some(SkillInstance::ACTIVE_PLAY_PHASE_TARGETS_STATE_TAG.as_slice())
            {
                skill.state_blob.clear();
            } else if skill.state_blob.get(..4)
                == Some(SkillInstance::TRACKED_PLAY_TARGETS_STATE_TAG.as_slice())
            {
                let cards = skill.tracked_cards();
                skill.state_blob = SkillInstance::TWKUJIAN_STATE_TAG.to_vec();
                for card in cards {
                    skill
                        .state_blob
                        .extend_from_slice(&card.raw().to_le_bytes());
                }
            }
        }
    }

    /// Next id.
    pub fn next_id(&self) -> u32 {
        self.next_id
    }
}
