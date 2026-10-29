//! 台服（tw*）技能实例的状态读写。

use super::*;

impl SkillInstance {
    pub(crate) fn reset_twchenxun_round(&mut self, round: u32) {
        self.state_blob = Self::TWCHENXUN_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob.extend_from_slice(&0_u32.to_le_bytes());
    }

    pub(crate) fn twchenxun_round(&self) -> u32 {
        if self.state_blob.get(..4) != Some(Self::TWCHENXUN_STATE_TAG.as_slice())
            || self.state_blob.len() < 12
        {
            return 0;
        }
        u32::from_le_bytes(self.state_blob[4..8].try_into().unwrap())
    }

    pub(crate) fn set_twchenxun_card(&mut self, card: CardId) {
        if self.state_blob.get(..4) != Some(Self::TWCHENXUN_STATE_TAG.as_slice())
            || self.state_blob.len() < 12
        {
            return;
        }
        self.state_blob[8..12].copy_from_slice(&card.raw().to_le_bytes());
    }

    pub(crate) fn twchenxun_card(&self) -> Option<CardId> {
        (self.state_blob.get(..4) == Some(Self::TWCHENXUN_STATE_TAG.as_slice())
            && self.state_blob.len() >= 12)
            .then(|| {
                CardId::new(u32::from_le_bytes(
                    self.state_blob[8..12].try_into().unwrap(),
                ))
            })
            .filter(|card| card.is_some())
    }

    pub(crate) fn clear_twchenxun_card(&mut self) {
        if self.twchenxun_card().is_some() {
            self.state_blob[8..12].fill(0);
        }
    }

    pub(crate) fn twchenxun_targets(&self) -> Vec<PlayerId> {
        if self.state_blob.get(..4) != Some(Self::TWCHENXUN_STATE_TAG.as_slice())
            || self.state_blob.len() < 12
            || (self.state_blob.len() - 12) % 4 != 0
        {
            return Vec::new();
        }
        self.state_blob[12..]
            .chunks_exact(4)
            .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .collect()
    }

    pub(crate) fn add_twchenxun_target(&mut self, target: PlayerId) {
        if self.state_blob.get(..4) == Some(Self::TWCHENXUN_STATE_TAG.as_slice())
            && self.state_blob.len() >= 12
            && !self.twchenxun_targets().contains(&target)
        {
            self.state_blob
                .extend_from_slice(&target.raw().to_le_bytes());
        }
    }

    pub(crate) fn twchayi_state(&self) -> Option<(u8, u32)> {
        (self.state_blob.len() == 9
            && self.state_blob.get(..4) == Some(Self::TWCHAYI_STATE_TAG.as_slice()))
        .then(|| {
            (
                self.state_blob[4],
                u32::from_le_bytes(self.state_blob[5..9].try_into().unwrap()),
            )
        })
    }

    pub(crate) fn set_twchayi_state(&mut self, branch: u8, hand_count: u32) {
        self.state_blob = Self::TWCHAYI_STATE_TAG.to_vec();
        self.state_blob.push(branch.min(1));
        self.state_blob.extend_from_slice(&hand_count.to_le_bytes());
    }

    pub(crate) fn twzhengjian_state(&self) -> (Option<u8>, bool, Vec<PlayerId>) {
        if self.state_blob.len() < 6
            || self.state_blob.get(..4) != Some(Self::TWZHENGJIAN_STATE_TAG.as_slice())
        {
            return (None, false, Vec::new());
        }
        let branch = (self.state_blob[4] <= 1).then_some(self.state_blob[4]);
        let contributors = self.state_blob[6..]
            .chunks_exact(4)
            .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .collect();
        (branch, self.state_blob[5] != 0, contributors)
    }

    pub(crate) fn set_twzhengjian_state(
        &mut self,
        branch: u8,
        awakened: bool,
        mut contributors: Vec<PlayerId>,
    ) {
        contributors.sort_unstable_by_key(|player| player.raw());
        contributors.dedup();
        self.state_blob = Self::TWZHENGJIAN_STATE_TAG.to_vec();
        self.state_blob.push(branch.min(1));
        self.state_blob.push(u8::from(awakened));
        for contributor in contributors {
            self.state_blob
                .extend_from_slice(&contributor.raw().to_le_bytes());
        }
    }

    pub(crate) fn twguoyi_choice_mask(&self) -> u8 {
        (self.state_blob.len() == 5
            && self.state_blob.get(..4) == Some(Self::TWGUOYI_STATE_TAG.as_slice()))
        .then(|| self.state_blob[4] & 0b11)
        .unwrap_or(0)
    }

    pub(crate) fn mark_twguoyi_choice(&mut self, choice: u8) -> u8 {
        let mask = self.twguoyi_choice_mask() | (1 << choice.min(1));
        self.state_blob = Self::TWGUOYI_STATE_TAG.to_vec();
        self.state_blob.push(mask);
        mask
    }

    pub(crate) fn clear_twguoyi_turn_state(&mut self) {
        if self.state_blob.get(..4) == Some(Self::TWGUOYI_STATE_TAG.as_slice()) {
            self.state_blob.clear();
        }
    }

    pub(crate) fn twfupan_target_state(&self, target: PlayerId) -> u8 {
        if self.state_blob.get(..4) != Some(Self::TWFUPAN_STATE_TAG.as_slice()) {
            return 0;
        }
        self.state_blob[4..]
            .chunks_exact(5)
            .find(|chunk| u32::from_le_bytes(chunk[..4].try_into().unwrap()) == target.raw())
            .map_or(0, |chunk| chunk[4].min(2))
    }

    pub(crate) fn set_twfupan_target_state(&mut self, target: PlayerId, state: u8) -> bool {
        if target.is_none()
            || !matches!(state, 1 | 2)
            || (!self.state_blob.is_empty()
                && self.state_blob.get(..4) != Some(Self::TWFUPAN_STATE_TAG.as_slice()))
        {
            return false;
        }
        let mut states: Vec<_> = self
            .state_blob
            .get(4..)
            .unwrap_or_default()
            .chunks_exact(5)
            .map(|chunk| {
                (
                    u32::from_le_bytes(chunk[..4].try_into().unwrap()),
                    chunk[4].min(2),
                )
            })
            .collect();
        if let Some((_, stored)) = states
            .iter_mut()
            .find(|(player, _)| *player == target.raw())
        {
            *stored = state;
        } else {
            states.push((target.raw(), state));
            states.sort_unstable_by_key(|(player, _)| *player);
        }
        self.state_blob = Self::TWFUPAN_STATE_TAG.to_vec();
        for (player, state) in states {
            self.state_blob.extend_from_slice(&player.to_le_bytes());
            self.state_blob.push(state);
        }
        true
    }

    pub(crate) fn twkujian_cards(&self) -> Vec<CardId> {
        if self.state_blob.get(..4) == Some(Self::TRACKED_PLAY_TARGETS_STATE_TAG.as_slice()) {
            let target_count = self
                .state_blob
                .get(4..8)
                .map(|raw| u32::from_le_bytes(raw.try_into().unwrap()) as usize)
                .unwrap_or(usize::MAX);
            let offset = 8usize.saturating_add(target_count.saturating_mul(4));
            return self
                .state_blob
                .get(offset..)
                .unwrap_or_default()
                .chunks_exact(4)
                .map(|chunk| CardId::new(u32::from_le_bytes(chunk.try_into().unwrap())))
                .filter(|card| card.is_some())
                .collect();
        }
        if self.state_blob.get(..4) != Some(Self::TWKUJIAN_STATE_TAG.as_slice()) {
            return Vec::new();
        }
        self.state_blob[4..]
            .chunks_exact(4)
            .map(|chunk| CardId::new(u32::from_le_bytes(chunk.try_into().unwrap())))
            .filter(|card| card.is_some())
            .collect()
    }

    pub(crate) fn add_twkujian_cards(&mut self, cards: &[CardId]) -> bool {
        if !self.state_blob.is_empty()
            && self.state_blob.get(..4) != Some(Self::TWKUJIAN_STATE_TAG.as_slice())
        {
            return false;
        }
        let mut tracked = self.twkujian_cards();
        tracked.extend(cards.iter().copied().filter(|card| card.is_some()));
        tracked.sort_unstable_by_key(|card| card.raw());
        tracked.dedup();
        self.state_blob = Self::TWKUJIAN_STATE_TAG.to_vec();
        for card in tracked {
            self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        }
        true
    }

    pub(crate) fn take_twkujian_cards(&mut self, cards: &[CardId]) -> Vec<CardId> {
        let mut tracked = self.twkujian_cards();
        let mut taken = Vec::new();
        tracked.retain(|card| {
            if cards.contains(card) {
                taken.push(*card);
                false
            } else {
                true
            }
        });
        self.state_blob = Self::TWKUJIAN_STATE_TAG.to_vec();
        for card in tracked {
            self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        }
        taken
    }

    pub(crate) fn twruilian_state(&self) -> (u32, PlayerId, u32, u8) {
        if self.state_blob.len() != 17
            || self.state_blob.get(..4) != Some(Self::TWRUILIAN_STATE_TAG.as_slice())
        {
            return (0, PlayerId::NONE, 0, 0);
        }
        (
            u32::from_le_bytes(self.state_blob[4..8].try_into().unwrap()),
            PlayerId::new(u32::from_le_bytes(
                self.state_blob[8..12].try_into().unwrap(),
            )),
            u32::from_le_bytes(self.state_blob[12..16].try_into().unwrap()),
            self.state_blob[16] & 0b111,
        )
    }

    pub(crate) fn set_twruilian_state(
        &mut self,
        round: u32,
        target: PlayerId,
        count: u32,
        mask: u8,
    ) {
        self.state_blob = Self::TWRUILIAN_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
        self.state_blob.extend_from_slice(&count.to_le_bytes());
        self.state_blob.push(mask & 0b111);
    }

    pub(crate) fn set_twruilian_prompt_round(&mut self, round: u32) {
        let (_, target, count, mask) = self.twruilian_state();
        self.set_twruilian_state(round, target, count, mask);
    }

    pub(crate) fn set_twruilian_target(&mut self, target: PlayerId) {
        let (round, _, _, _) = self.twruilian_state();
        self.set_twruilian_state(round, target, 0, 0);
    }

    pub(crate) fn record_twruilian_discard(&mut self, target: PlayerId, count: u32, mask: u8) {
        let (round, selected, previous_count, previous_mask) = self.twruilian_state();
        if selected == target && count > 0 {
            self.set_twruilian_state(
                round,
                selected,
                previous_count.saturating_add(count),
                previous_mask | mask,
            );
        }
    }

    pub(crate) fn take_twruilian_summary(&mut self, target: PlayerId) -> Option<(u32, u8)> {
        let (round, selected, count, mask) = self.twruilian_state();
        if selected != target {
            return None;
        }
        self.set_twruilian_state(round, PlayerId::NONE, 0, 0);
        Some((count, mask))
    }

    pub(crate) fn set_twjianwei_compare(&mut self, actor: PlayerId, target: PlayerId) {
        self.state_blob = Self::TWJIANWEI_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&actor.raw().to_le_bytes());
        self.state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
    }

    pub(crate) fn twjianwei_compare(&self) -> Option<(PlayerId, PlayerId)> {
        (self.state_blob.len() == 12
            && self.state_blob.get(..4) == Some(Self::TWJIANWEI_STATE_TAG.as_slice()))
        .then(|| {
            (
                PlayerId::new(u32::from_le_bytes(
                    self.state_blob[4..8].try_into().unwrap(),
                )),
                PlayerId::new(u32::from_le_bytes(
                    self.state_blob[8..12].try_into().unwrap(),
                )),
            )
        })
    }

    pub(crate) fn clear_twjianwei_compare(&mut self) {
        if self.twjianwei_compare().is_some() {
            self.state_blob.clear();
        }
    }

    pub(crate) fn set_twzhibian_compare(&mut self, target: PlayerId) {
        self.state_blob = Self::TWZHIBIAN_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
    }

    pub(crate) fn twzhibian_compare(&self) -> Option<PlayerId> {
        (self.state_blob.len() == 8
            && self.state_blob.get(..4) == Some(Self::TWZHIBIAN_STATE_TAG.as_slice()))
        .then(|| {
            PlayerId::new(u32::from_le_bytes(
                self.state_blob[4..8].try_into().unwrap(),
            ))
        })
    }

    pub(crate) fn clear_twzhibian_compare(&mut self) {
        if self.twzhibian_compare().is_some() {
            self.state_blob.clear();
        }
    }
}
