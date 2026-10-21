//! Runtime skill state for `content_flag` and related helpers.

use super::*;

impl SkillInstance {
    pub(crate) fn content_flag(&self, bit: u8) -> Option<bool> {
        (bit < 64).then(|| self.content_flags & (1u64 << bit) != 0)
    }

    pub(crate) fn set_content_flag(&mut self, bit: u8, value: bool) -> bool {
        if bit >= 64 {
            return false;
        }
        if value {
            self.content_flags |= 1u64 << bit;
        } else {
            self.content_flags &= !(1u64 << bit);
        }
        true
    }

    pub(crate) const CHEGU_STATE_TAG: [u8; 4] = *b"CGU1";

    pub(crate) const OLDJIEFAN_STATE_TAG: [u8; 4] = *b"JFN1";

    pub(crate) const OLDHUISHENG_STATE_TAG: [u8; 4] = *b"HSH1";

    pub(crate) const XINYAOMING_STATE_TAG: [u8; 4] = *b"XYM1";

    pub(crate) const JIAOHUA_STATE_TAG: [u8; 4] = *b"JHW1";

    pub(crate) const LUANQUN_HELPER_STATE_TAG: [u8; 4] = *b"LQH1";

    pub(crate) const MBJINZU_HELPER_STATE_TAG: [u8; 4] = *b"JZU1";

    pub(crate) const TWFUPAN_STATE_TAG: [u8; 4] = *b"FPN1";

    pub(crate) const TWGUOYI_STATE_TAG: [u8; 4] = *b"GYI1";

    pub(crate) const TWKUJIAN_STATE_TAG: [u8; 4] = *b"KJN1";

    pub(crate) const TRACKED_PLAY_TARGETS_STATE_TAG: [u8; 4] = *b"TPC1";

    pub(crate) const TWRUILIAN_STATE_TAG: [u8; 4] = *b"RLI1";

    pub(crate) const TWJIANWEI_STATE_TAG: [u8; 4] = *b"JNW1";

    pub(crate) const TWZHIBIAN_STATE_TAG: [u8; 4] = *b"ZHB1";

    pub(crate) const ZHIBA_STATE_TAG: [u8; 4] = *b"ZHB2";

    pub(crate) const TWCHAYI_STATE_TAG: [u8; 4] = *b"CHY1";

    pub(crate) const TWZHENGJIAN_STATE_TAG: [u8; 4] = *b"ZJN1";

    pub(crate) const TWCHENXUN_STATE_TAG: [u8; 4] = *b"CXN1";

    pub(crate) const POTHAOSHI_STATE_TAG: [u8; 4] = *b"PTH1";

    pub(crate) const POTJIEJIE_STATE_TAG: [u8; 4] = *b"PJJ1";

    pub(crate) const POTHUILV_STATE_TAG: [u8; 4] = *b"PHL1";

    pub(crate) const POTSONGYAN_STATE_TAG: [u8; 4] = *b"PSY1";

    pub(crate) const POTSHISHI_STATE_TAG: [u8; 4] = *b"PSS1";

    pub(crate) const POTCAOSHUANG_STATE_TAG: [u8; 4] = *b"PCS1";

    pub(crate) const POTNIZUN_STATE_TAG: [u8; 4] = *b"PNZ1";

    pub(crate) const POTDAOZHUAN_STATE_TAG: [u8; 4] = *b"PDZ1";

    pub(crate) const DDD_CHASHI_STATE_TAG: [u8; 4] = *b"DCS1";

    pub(crate) const DDD_ZHUANSHE_STATE_TAG: [u8; 4] = *b"DZS1";

    pub(crate) const DDD_ANZHI_STATE_TAG: [u8; 4] = *b"DAZ1";

    pub(crate) const DDD_SHIXING_STATE_TAG: [u8; 4] = *b"DSX1";

    pub(crate) fn ddd_shixing_grants(&self) -> Vec<SkillId> {
        if self.state_blob.len() < 4
            || self.state_blob.get(..4) != Some(Self::DDD_SHIXING_STATE_TAG.as_slice())
            || (self.state_blob.len() - 4) % 4 != 0
        {
            return Vec::new();
        }
        self.state_blob[4..]
            .chunks_exact(4)
            .map(|entry| SkillId::new(u32::from_le_bytes(entry.try_into().unwrap())))
            .collect()
    }

    pub(crate) fn set_ddd_shixing_grants(&mut self, grants: &[SkillId]) {
        self.state_blob = Self::DDD_SHIXING_STATE_TAG.to_vec();
        for skill in grants {
            self.state_blob
                .extend_from_slice(&skill.raw().to_le_bytes());
        }
    }

    pub(crate) fn ddd_anzhi_state(&self) -> (bool, PlayerId, PlayerId) {
        if self.state_blob.len() != 13
            || self.state_blob.get(..4) != Some(Self::DDD_ANZHI_STATE_TAG.as_slice())
        {
            return (false, PlayerId::NONE, PlayerId::NONE);
        }
        (
            self.state_blob[4] != 0,
            PlayerId::new(u32::from_le_bytes(
                self.state_blob[5..9].try_into().unwrap(),
            )),
            PlayerId::new(u32::from_le_bytes(
                self.state_blob[9..13].try_into().unwrap(),
            )),
        )
    }

    pub(crate) fn set_ddd_anzhi_state(&mut self, black: bool, actor: PlayerId, target: PlayerId) {
        self.state_blob = Self::DDD_ANZHI_STATE_TAG.to_vec();
        self.state_blob.push(u8::from(black));
        self.state_blob
            .extend_from_slice(&actor.raw().to_le_bytes());
        self.state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
    }

    pub(crate) fn record_ddd_anzhi_color(&mut self, black: bool) {
        let (_, actor, target) = self.ddd_anzhi_state();
        self.set_ddd_anzhi_state(black, actor, target);
    }

    pub(crate) fn arm_ddd_anzhi(&mut self, actor: PlayerId, target: PlayerId) {
        let (black, _, _) = self.ddd_anzhi_state();
        self.set_ddd_anzhi_state(black, actor, target);
    }

    pub(crate) fn clear_ddd_anzhi_target(&mut self) {
        let (black, _, _) = self.ddd_anzhi_state();
        self.set_ddd_anzhi_state(black, PlayerId::NONE, PlayerId::NONE);
    }

    pub(crate) fn ddd_zhuanshe_state(&self) -> Option<(PlayerId, Vec<(u8, u32, bool)>)> {
        if self.state_blob.len() < 8
            || self.state_blob.get(..4) != Some(Self::DDD_ZHUANSHE_STATE_TAG.as_slice())
            || (self.state_blob.len() - 8) % 6 != 0
        {
            return None;
        }
        let actor = PlayerId::new(u32::from_le_bytes(
            self.state_blob[4..8].try_into().unwrap(),
        ));
        let names = self.state_blob[8..]
            .chunks_exact(6)
            .map(|entry| {
                (
                    entry[0],
                    u32::from_le_bytes(entry[1..5].try_into().unwrap()),
                    entry[5] != 0,
                )
            })
            .collect();
        Some((actor, names))
    }

    pub(crate) fn record_ddd_zhuanshe_name(&mut self, actor: PlayerId, kind: u8, def: u32) {
        let mut names = self
            .ddd_zhuanshe_state()
            .filter(|(current, _)| *current == actor)
            .map(|(_, names)| names)
            .unwrap_or_default();
        if !names.iter().any(|(k, d, _)| *k == kind && *d == def) {
            names.push((kind, def, false));
        }
        self.state_blob = Self::DDD_ZHUANSHE_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&actor.raw().to_le_bytes());
        for (kind, def, used) in names {
            self.state_blob.push(kind);
            self.state_blob.extend_from_slice(&def.to_le_bytes());
            self.state_blob.push(u8::from(used));
        }
    }

    pub(crate) fn mark_ddd_zhuanshe_used(&mut self, actor: PlayerId, kind: u8, def: u32) -> bool {
        let Some((stored_actor, mut names)) = self.ddd_zhuanshe_state() else {
            return false;
        };
        if stored_actor != actor {
            return false;
        }
        let mut matched = false;
        for (stored_kind, stored_def, used) in &mut names {
            if *stored_kind == kind && *stored_def == def {
                *used = true;
                matched = true;
            }
        }
        if matched {
            self.state_blob = Self::DDD_ZHUANSHE_STATE_TAG.to_vec();
            self.state_blob
                .extend_from_slice(&actor.raw().to_le_bytes());
            for (kind, def, used) in names {
                self.state_blob.push(kind);
                self.state_blob.extend_from_slice(&def.to_le_bytes());
                self.state_blob.push(u8::from(used));
            }
        }
        matched
    }

    pub(crate) fn clear_ddd_zhuanshe_state(&mut self) {
        if self.ddd_zhuanshe_state().is_some() {
            self.state_blob.clear();
        }
    }

    pub(crate) fn ddd_chashi_state(&self) -> Option<(PlayerId, u8, u8)> {
        (self.state_blob.len() == 10
            && self.state_blob.get(..4) == Some(Self::DDD_CHASHI_STATE_TAG.as_slice()))
        .then(|| {
            (
                PlayerId::new(u32::from_le_bytes(
                    self.state_blob[4..8].try_into().unwrap(),
                )),
                self.state_blob[8],
                self.state_blob[9],
            )
        })
    }

    pub(crate) fn set_ddd_chashi_state(&mut self, target: PlayerId, suit: u8, category: u8) {
        self.state_blob = Self::DDD_CHASHI_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
        self.state_blob.push(suit);
        self.state_blob.push(category);
    }

    pub(crate) fn clear_ddd_chashi_state(&mut self) {
        if self.ddd_chashi_state().is_some() {
            self.state_blob.clear();
        }
    }

    /// Decode one 典易/奢权 instance's assigned timings and current-round progress.
    pub(crate) fn potcaoshuang_state(&self, round: u32) -> (u8, u8, u32) {
        if self.state_blob.len() != 14
            || self.state_blob.get(..4) != Some(Self::POTCAOSHUANG_STATE_TAG.as_slice())
        {
            return (0, 0, 0);
        }
        let assigned = self.state_blob[4] & 0b111;
        if u32::from_le_bytes(self.state_blob[5..9].try_into().unwrap()) != round {
            return (assigned, 0, 0);
        }
        (
            assigned,
            self.state_blob[9] & 0b111,
            u32::from_le_bytes(self.state_blob[10..14].try_into().unwrap()),
        )
    }

    pub(crate) fn set_potcaoshuang_state(
        &mut self,
        round: u32,
        assigned: u8,
        triggered: u8,
        gained: u32,
    ) {
        self.state_blob = Self::POTCAOSHUANG_STATE_TAG.to_vec();
        self.state_blob.push(assigned & 0b111);
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob.push(triggered & 0b111);
        self.state_blob.extend_from_slice(&gained.to_le_bytes());
    }

    pub(crate) fn assign_potcaoshuang_timing(&mut self, round: u32, timing: u8) -> bool {
        let (assigned, triggered, gained) = self.potcaoshuang_state(round);
        if timing & 0b111 == 0 || assigned & timing != 0 {
            return false;
        }
        self.set_potcaoshuang_state(round, assigned | timing, triggered, gained);
        true
    }

    /// Mark one assigned timing fired this round and add its actual gained-card count.
    pub(crate) fn fire_potcaoshuang_timing(
        &mut self,
        round: u32,
        timing: u8,
        gained_delta: u32,
    ) -> Option<(u32, u32)> {
        let (assigned, triggered, gained) = self.potcaoshuang_state(round);
        if assigned & timing == 0 || triggered & timing != 0 {
            return None;
        }
        let triggered = triggered | timing;
        let gained = gained.saturating_add(gained_delta);
        self.set_potcaoshuang_state(round, assigned, triggered, gained);
        Some((triggered.count_ones(), gained))
    }

    pub(crate) fn potnizun_cards(&self, round: u32) -> Vec<CardId> {
        if self.state_blob.len() < 8
            || (self.state_blob.len() - 8) % 4 != 0
            || self.state_blob.get(..4) != Some(Self::POTNIZUN_STATE_TAG.as_slice())
            || u32::from_le_bytes(self.state_blob[4..8].try_into().unwrap()) != round
        {
            return Vec::new();
        }
        self.state_blob[8..]
            .chunks_exact(4)
            .map(|chunk| CardId::new(u32::from_le_bytes(chunk.try_into().unwrap())))
            .filter(|card| card.is_some())
            .collect()
    }

    pub(crate) fn record_potnizun_card(&mut self, round: u32, card: CardId) -> bool {
        let mut cards = self.potnizun_cards(round);
        if card.is_none() || cards.contains(&card) {
            return false;
        }
        cards.push(card);
        cards.sort_by_key(|card| card.raw());
        self.state_blob = Self::POTNIZUN_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        for card in cards {
            self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        }
        true
    }

    pub(crate) fn potnizun_use_count(&self, round: u32) -> u32 {
        self.potnizun_cards(round).len() as u32
    }

    pub(crate) fn potdaozhuan_state(&self, round: u32) -> (u32, u32, PlayerId, bool) {
        if self.state_blob.len() != 21
            || self.state_blob.get(..4) != Some(Self::POTDAOZHUAN_STATE_TAG.as_slice())
            || u32::from_le_bytes(self.state_blob[4..8].try_into().unwrap()) != round
        {
            return (0, 0, PlayerId::NONE, false);
        }
        (
            u32::from_le_bytes(self.state_blob[8..12].try_into().unwrap()),
            u32::from_le_bytes(self.state_blob[12..16].try_into().unwrap()),
            PlayerId::new(u32::from_le_bytes(
                self.state_blob[16..20].try_into().unwrap(),
            )),
            self.state_blob[20] != 0,
        )
    }

    pub(crate) fn potdaozhuan_available(
        &self,
        round: u32,
        _current: PlayerId,
        name_bit: u32,
    ) -> bool {
        let (used_names, _, _, banned) = self.potdaozhuan_state(round);
        !banned && used_names & name_bit == 0 && self.turn_fire_count == 0
    }

    pub(crate) fn record_potdaozhuan(
        &mut self,
        round: u32,
        current: PlayerId,
        name_bit: u32,
        banned: bool,
    ) {
        let (used_names, _, _, already_banned) = self.potdaozhuan_state(round);
        self.state_blob = Self::POTDAOZHUAN_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob
            .extend_from_slice(&(used_names | name_bit).to_le_bytes());
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob
            .extend_from_slice(&current.raw().to_le_bytes());
        self.state_blob.push(u8::from(banned || already_banned));
        self.turn_fire_count = 1;
    }

    /// Decode 隳律's round colors and the one exact card pending for a Play phase.
    pub(crate) fn pothuilv_state(&self, round: u32) -> (u8, PlayerId, engine_api::CardId, bool) {
        if self.state_blob.len() != 18
            || self.state_blob.get(..4) != Some(Self::POTHUILV_STATE_TAG.as_slice())
            || u32::from_le_bytes(self.state_blob[4..8].try_into().unwrap()) != round
        {
            return (0, PlayerId::NONE, engine_api::CardId::NONE, false);
        }
        (
            self.state_blob[8] & 0b11,
            PlayerId::new(u32::from_le_bytes(
                self.state_blob[9..13].try_into().unwrap(),
            )),
            engine_api::CardId::new(u32::from_le_bytes(
                self.state_blob[13..17].try_into().unwrap(),
            )),
            self.state_blob[17] != 0,
        )
    }

    pub(crate) fn set_pothuilv_state(
        &mut self,
        round: u32,
        colors: u8,
        actor: PlayerId,
        card: engine_api::CardId,
        used: bool,
    ) {
        self.state_blob = Self::POTHUILV_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob.push(colors & 0b11);
        self.state_blob
            .extend_from_slice(&actor.raw().to_le_bytes());
        self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        self.state_blob.push(u8::from(used));
    }

    pub(crate) fn potsongyan_state(
        &self,
        round: u32,
    ) -> (Vec<(PlayerId, u8)>, Vec<engine_api::CardId>) {
        if self.state_blob.len() < 10
            || self.state_blob.get(..4) != Some(Self::POTSONGYAN_STATE_TAG.as_slice())
            || u32::from_le_bytes(self.state_blob[4..8].try_into().unwrap()) != round
        {
            return (Vec::new(), Vec::new());
        }
        let count = self.state_blob[8] as usize;
        let cards_count_at = 9usize.saturating_add(count.saturating_mul(5));
        let Some(&card_count) = self.state_blob.get(cards_count_at) else {
            return (Vec::new(), Vec::new());
        };
        if self.state_blob.len() != cards_count_at + 1 + usize::from(card_count) * 4 {
            return (Vec::new(), Vec::new());
        }
        let entries = self.state_blob[9..cards_count_at]
            .chunks_exact(5)
            .map(|entry| {
                (
                    PlayerId::new(u32::from_le_bytes(entry[..4].try_into().unwrap())),
                    entry[4] & 0b11,
                )
            })
            .filter(|(player, mask)| player.is_some() && *mask != 0)
            .collect();
        let cards = self.state_blob[cards_count_at + 1..]
            .chunks_exact(4)
            .map(|entry| engine_api::CardId::new(u32::from_le_bytes(entry.try_into().unwrap())))
            .filter(|card| card.is_some())
            .collect();
        (entries, cards)
    }

    pub(crate) fn set_potsongyan_state(
        &mut self,
        round: u32,
        mut entries: Vec<(PlayerId, u8)>,
        mut cards: Vec<engine_api::CardId>,
    ) {
        entries.sort_by_key(|(player, _)| player.raw());
        entries.dedup_by_key(|(player, _)| player.raw());
        cards.sort_by_key(|card| card.raw());
        cards.dedup();
        entries.truncate(u8::MAX as usize);
        cards.truncate(u8::MAX as usize);
        self.state_blob = Self::POTSONGYAN_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob.push(entries.len() as u8);
        for (player, mask) in entries {
            self.state_blob
                .extend_from_slice(&player.raw().to_le_bytes());
            self.state_blob.push(mask & 0b11);
        }
        self.state_blob.push(cards.len() as u8);
        for card in cards {
            self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
        }
    }

    pub(crate) fn record_potsongyan_discard(&mut self, round: u32, player: PlayerId, colors: u8) {
        if player.is_none() || colors & 0b11 == 0 {
            return;
        }
        let (mut entries, cards) = self.potsongyan_state(round);
        if let Some((_, mask)) = entries.iter_mut().find(|(found, _)| *found == player) {
            *mask |= colors & 0b11;
        } else {
            entries.push((player, colors & 0b11));
        }
        self.set_potsongyan_state(round, entries, cards);
    }

    pub(crate) fn add_potsongyan_card(&mut self, round: u32, card: engine_api::CardId) {
        let (entries, mut cards) = self.potsongyan_state(round);
        cards.push(card);
        self.set_potsongyan_state(round, entries, cards);
    }

    pub(crate) fn remove_potsongyan_card(&mut self, round: u32, card: engine_api::CardId) {
        let (entries, mut cards) = self.potsongyan_state(round);
        cards.retain(|found| *found != card);
        self.set_potsongyan_state(round, entries, cards);
    }

    pub(crate) fn potsongyan_response_colors(
        &self,
        round: u32,
        card: engine_api::CardId,
        player: PlayerId,
    ) -> u8 {
        let (entries, cards) = self.potsongyan_state(round);
        if !cards.contains(&card) {
            return 0;
        }
        entries
            .into_iter()
            .find_map(|(found, mask)| (found == player).then_some(mask))
            .unwrap_or(0)
    }

    pub(crate) fn potshishi_state(&self) -> (bool, u32, Vec<(engine_api::CardId, u32)>) {
        if self.state_blob.len() < 9
            || (self.state_blob.len() - 9) % 8 != 0
            || self.state_blob.get(..4) != Some(Self::POTSHISHI_STATE_TAG.as_slice())
        {
            return (false, 0, Vec::new());
        }
        let disabled = self.state_blob[4] != 0;
        let sources = u32::from_le_bytes(self.state_blob[5..9].try_into().unwrap());
        let cards = self.state_blob[9..]
            .chunks_exact(8)
            .map(|entry| {
                (
                    engine_api::CardId::new(u32::from_le_bytes(entry[..4].try_into().unwrap())),
                    u32::from_le_bytes(entry[4..].try_into().unwrap()),
                )
            })
            .filter(|(card, mask)| card.is_some() && *mask != 0)
            .collect();
        (disabled, sources, cards)
    }

    pub(crate) fn set_potshishi_state(
        &mut self,
        disabled: bool,
        sources: u32,
        mut cards: Vec<(engine_api::CardId, u32)>,
    ) {
        cards.sort_by_key(|(card, _)| card.raw());
        cards.dedup_by_key(|(card, _)| card.raw());
        self.state_blob = Self::POTSHISHI_STATE_TAG.to_vec();
        self.state_blob.push(u8::from(disabled));
        self.state_blob.extend_from_slice(&sources.to_le_bytes());
        for (card, mask) in cards {
            self.state_blob.extend_from_slice(&card.raw().to_le_bytes());
            self.state_blob.extend_from_slice(&mask.to_le_bytes());
        }
    }

    pub(crate) fn record_potshishi_source(&mut self, source: PlayerId) {
        let (disabled, mut sources, cards) = self.potshishi_state();
        if disabled || !(1..=32).contains(&source.raw()) {
            return;
        }
        sources |= 1 << (source.raw() - 1);
        self.set_potshishi_state(false, sources, cards);
    }

    pub(crate) fn snapshot_potshishi_card(&mut self, card: engine_api::CardId) {
        let (disabled, sources, mut cards) = self.potshishi_state();
        if disabled || sources == 0 || card.is_none() {
            return;
        }
        if let Some((_, mask)) = cards.iter_mut().find(|(found, _)| *found == card) {
            *mask |= sources;
        } else {
            cards.push((card, sources));
        }
        self.set_potshishi_state(false, sources, cards);
    }

    pub(crate) fn disable_potshishi(&mut self) {
        let (_, _, cards) = self.potshishi_state();
        self.set_potshishi_state(true, 0, cards);
    }

    pub(crate) fn clear_potshishi_card(&mut self, card: engine_api::CardId) {
        let (disabled, sources, mut cards) = self.potshishi_state();
        cards.retain(|(found, _)| *found != card);
        self.set_potshishi_state(disabled, sources, cards);
    }

    pub(crate) fn potshishi_blocks(&self, card: engine_api::CardId, responder: PlayerId) -> bool {
        if !(1..=32).contains(&responder.raw()) {
            return false;
        }
        let (_, _, cards) = self.potshishi_state();
        cards
            .into_iter()
            .any(|(found, mask)| found == card && mask & (1 << (responder.raw() - 1)) != 0)
    }

    pub(crate) fn potjiejie_state(&self, round: u32) -> (u8, Vec<(PlayerId, u8)>) {
        if self.state_blob.len() < 9
            || self.state_blob.get(..4) != Some(Self::POTJIEJIE_STATE_TAG.as_slice())
            || (self.state_blob.len() - 9) % 5 != 0
            || u32::from_le_bytes(self.state_blob[4..8].try_into().unwrap()) != round
        {
            return (0, Vec::new());
        }
        let entries: Vec<_> = self.state_blob[9..]
            .chunks_exact(5)
            .map(|entry| {
                (
                    PlayerId::new(u32::from_le_bytes(entry[..4].try_into().unwrap())),
                    entry[4] & 0x0f,
                )
            })
            .collect();
        if entries.iter().any(|(player, _)| player.is_none())
            || !entries
                .windows(2)
                .all(|pair| pair[0].0.raw() < pair[1].0.raw())
        {
            return (0, Vec::new());
        }
        (self.state_blob[8].min(2), entries)
    }

    /// Record one actor's union of hand suits for this round and consume one of the two
    /// unique-maximum 清识 bonuses when earned.
    pub(crate) fn record_potjiejie_suits(
        &mut self,
        round: u32,
        actor: PlayerId,
        suits: u8,
        alive_players: &[PlayerId],
    ) -> bool {
        let (mut bonus_count, mut entries) = self.potjiejie_state(round);
        if let Some((_, mask)) = entries.iter_mut().find(|(player, _)| *player == actor) {
            *mask |= suits & 0x0f;
        } else {
            entries.push((actor, suits & 0x0f));
            entries.sort_by_key(|(player, _)| player.raw());
        }
        let actor_count = entries
            .iter()
            .find(|(player, _)| *player == actor)
            .map(|(_, mask)| mask.count_ones())
            .unwrap_or(0);
        let earned = bonus_count < 2
            && alive_players
                .iter()
                .filter(|player| **player != actor)
                .all(|player| {
                    entries
                        .iter()
                        .find(|(recorded, _)| recorded == player)
                        .map(|(_, mask)| mask.count_ones())
                        .unwrap_or(0)
                        < actor_count
                });
        if earned {
            bonus_count += 1;
        }
        self.state_blob = Self::POTJIEJIE_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob.push(bonus_count);
        for (player, mask) in entries {
            self.state_blob
                .extend_from_slice(&player.raw().to_le_bytes());
            self.state_blob.push(mask);
        }
        earned
    }

    /// Authorized 好施 borrowers and their remaining empty-hand refills.
    pub(crate) fn pothaoshi_grants(&self) -> Option<Vec<(PlayerId, u8)>> {
        if self.state_blob.is_empty() {
            return Some(Vec::new());
        }
        if self.state_blob.len() < 12
            || (self.state_blob.len() - 4) % 8 != 0
            || self.state_blob.get(..4) != Some(Self::POTHAOSHI_STATE_TAG.as_slice())
        {
            return None;
        }
        let grants: Vec<_> = self.state_blob[4..]
            .chunks_exact(8)
            .map(|entry| {
                (
                    PlayerId::new(u32::from_le_bytes(entry[..4].try_into().unwrap())),
                    u32::from_le_bytes(entry[4..].try_into().unwrap()) as u8,
                )
            })
            .collect();
        grants
            .iter()
            .all(|(target, remaining)| target.is_some() && *remaining <= 2)
            .then_some(grants)
            .filter(|grants| {
                grants
                    .windows(2)
                    .all(|pair| pair[0].0.raw() < pair[1].0.raw())
            })
    }

    pub(crate) fn set_pothaoshi_grant(&mut self, target: PlayerId) -> bool {
        if target.is_none() {
            return false;
        }
        let Some(mut grants) = self.pothaoshi_grants() else {
            return false;
        };
        if let Some((_, remaining)) = grants.iter_mut().find(|(found, _)| *found == target) {
            *remaining = 2;
        } else {
            grants.push((target, 2));
            grants.sort_by_key(|(target, _)| target.raw());
        }
        self.state_blob = Self::POTHAOSHI_STATE_TAG.to_vec();
        for (target, remaining) in grants {
            self.state_blob
                .extend_from_slice(&target.raw().to_le_bytes());
            self.state_blob
                .extend_from_slice(&u32::from(remaining).to_le_bytes());
        }
        true
    }

    pub(crate) fn consume_pothaoshi_refill(&mut self, target: PlayerId) -> bool {
        let Some(mut grants) = self.pothaoshi_grants() else {
            return false;
        };
        let Some((_, remaining)) = grants.iter_mut().find(|(found, _)| *found == target) else {
            return false;
        };
        if *remaining == 0 {
            return false;
        }
        *remaining -= 1;
        self.state_blob = Self::POTHAOSHI_STATE_TAG.to_vec();
        for (target, remaining) in grants {
            self.state_blob
                .extend_from_slice(&target.raw().to_le_bytes());
            self.state_blob
                .extend_from_slice(&u32::from(remaining).to_le_bytes());
        }
        true
    }

    pub(crate) fn clear_pothaoshi_grants(&mut self) -> bool {
        if self.state_blob.get(..4) != Some(Self::POTHAOSHI_STATE_TAG.as_slice()) {
            return false;
        }
        self.state_blob.clear();
        true
    }

    pub(crate) fn chegu_bonus(&self, round: u32, current: PlayerId) -> u32 {
        if self.state_blob.get(..4) != Some(Self::CHEGU_STATE_TAG.as_slice())
            || self.state_blob.len() != 16
        {
            return 0;
        }
        let stored_round =
            u32::from_le_bytes(self.state_blob[4..8].try_into().expect("checked length"));
        let stored_current = PlayerId::new(u32::from_le_bytes(
            self.state_blob[8..12].try_into().unwrap(),
        ));
        (stored_round == round && stored_current == current)
            .then(|| u32::from_le_bytes(self.state_blob[12..16].try_into().unwrap()))
            .unwrap_or(0)
    }

    pub(crate) fn add_chegu_bonus(&mut self, round: u32, current: PlayerId) {
        let bonus = self.chegu_bonus(round, current).saturating_add(1);
        self.state_blob = Self::CHEGU_STATE_TAG.to_vec();
        self.state_blob.extend_from_slice(&round.to_le_bytes());
        self.state_blob
            .extend_from_slice(&current.raw().to_le_bytes());
        self.state_blob.extend_from_slice(&bonus.to_le_bytes());
    }

    pub(crate) fn set_source_free_physical_card(&mut self, card: CardId) {
        self.source_free_physical_card = card;
    }

    pub(crate) fn source_free_physical_card(&self) -> Option<CardId> {
        self.source_free_physical_card
            .is_some()
            .then_some(self.source_free_physical_card)
    }

    pub(crate) fn take_source_free_physical_card(&mut self) -> Option<CardId> {
        let card = self.source_free_physical_card();
        self.source_free_physical_card = CardId::NONE;
        card
    }

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

    pub(crate) fn oldjiefan_dying(&self) -> Option<PlayerId> {
        (self.state_blob.get(..4) == Some(Self::OLDJIEFAN_STATE_TAG.as_slice()))
            .then(|| self.state_blob.get(4..8))
            .flatten()
            .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
    }

    pub(crate) fn set_oldjiefan_dying(&mut self, dying: PlayerId) {
        self.state_blob = Self::OLDJIEFAN_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&dying.raw().to_le_bytes());
    }

    pub(crate) fn oldhuisheng_sources(&self) -> Vec<PlayerId> {
        if self.state_blob.get(..4) != Some(Self::OLDHUISHENG_STATE_TAG.as_slice()) {
            return Vec::new();
        }
        self.state_blob[4..]
            .chunks_exact(4)
            .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .collect()
    }

    pub(crate) fn add_oldhuisheng_source(&mut self, source: PlayerId) {
        let mut sources = self.oldhuisheng_sources();
        if !sources.contains(&source) {
            sources.push(source);
            sources.sort_by_key(|player| player.raw());
        }
        self.state_blob = Self::OLDHUISHENG_STATE_TAG.to_vec();
        for source in sources {
            self.state_blob
                .extend_from_slice(&source.raw().to_le_bytes());
        }
    }

    pub(crate) fn xinyaoming_used_mask(&self) -> u8 {
        (self.state_blob.len() == 5
            && self.state_blob.get(..4) == Some(Self::XINYAOMING_STATE_TAG.as_slice()))
        .then(|| self.state_blob[4])
        .unwrap_or(0)
    }

    pub(crate) fn mark_xinyaoming_action(&mut self, action: u8) {
        let bit = 1u8 << action.saturating_sub(1);
        let used_mask = self.xinyaoming_used_mask();
        self.state_blob = Self::XINYAOMING_STATE_TAG.to_vec();
        self.state_blob.push(used_mask | bit);
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

    pub(crate) fn clear_xinyaoming_turn_state(&mut self) {
        if self.state_blob.get(..4) == Some(Self::XINYAOMING_STATE_TAG.as_slice()) {
            self.state_blob.clear();
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

    pub(crate) fn jiaohua_category_mask(&self) -> u8 {
        (self.state_blob.len() == 5
            && self.state_blob.get(..4) == Some(Self::JIAOHUA_STATE_TAG.as_slice()))
        .then(|| self.state_blob[4] & 0b111)
        .unwrap_or(0)
    }

    pub(crate) fn record_jiaohua_category(&mut self, category: u8) -> bool {
        if !matches!(category, 0b001 | 0b010 | 0b100)
            || (!self.state_blob.is_empty()
                && self.state_blob.get(..4) != Some(Self::JIAOHUA_STATE_TAG.as_slice()))
        {
            return false;
        }
        let current = self.jiaohua_category_mask();
        if current & category != 0 {
            return false;
        }
        let next = current | category;
        if next == 0b111 {
            self.state_blob.clear();
        } else {
            self.state_blob = Self::JIAOHUA_STATE_TAG.to_vec();
            self.state_blob.push(next);
        }
        true
    }

    /// Decode one Luanqun helper's behavior and canonical source set.
    pub(crate) fn luanqun_helper(&self) -> Option<(u8, Vec<PlayerId>)> {
        if self.state_blob.len() < 9
            || (self.state_blob.len() - 5) % 4 != 0
            || self.state_blob.get(..4) != Some(Self::LUANQUN_HELPER_STATE_TAG.as_slice())
            || !matches!(self.state_blob[4], 1 | 2)
        {
            return None;
        }
        let mut sources = Vec::new();
        for chunk in self.state_blob[5..].chunks_exact(4) {
            let source = PlayerId::new(u32::from_le_bytes(chunk.try_into().ok()?));
            if source.is_none() {
                return None;
            }
            sources.push(source);
        }
        sources.sort_by_key(|source| source.raw());
        sources.dedup();
        (!sources.is_empty()).then_some((self.state_blob[4], sources))
    }

    /// Add one Luanqun source without losing prior owners of the same helper.
    pub(crate) fn add_luanqun_source(&mut self, behavior: u8, source: PlayerId) -> bool {
        if !matches!(behavior, 1 | 2) || source.is_none() {
            return false;
        }
        let mut sources = self
            .luanqun_helper()
            .filter(|(stored, _)| *stored == behavior)
            .map(|(_, sources)| sources)
            .unwrap_or_default();
        if !sources.contains(&source) {
            sources.push(source);
            sources.sort_by_key(|source| source.raw());
        }
        self.state_blob = Self::LUANQUN_HELPER_STATE_TAG.to_vec();
        self.state_blob.push(behavior);
        for source in sources {
            self.state_blob
                .extend_from_slice(&source.raw().to_le_bytes());
        }
        true
    }

    pub(crate) fn mbjinzu_targets(&self) -> Vec<(PlayerId, u32)> {
        if self.state_blob.len() < 12
            || (self.state_blob.len() - 4) % 8 != 0
            || self.state_blob.get(..4) != Some(Self::MBJINZU_HELPER_STATE_TAG.as_slice())
        {
            return Vec::new();
        }
        self.state_blob[4..]
            .chunks_exact(8)
            .filter_map(|chunk| {
                let target = PlayerId::new(u32::from_le_bytes(chunk[..4].try_into().ok()?));
                let amount = u32::from_le_bytes(chunk[4..].try_into().ok()?);
                (target.is_some() && amount > 0).then_some((target, amount))
            })
            .collect()
    }

    pub(crate) fn add_mbjinzu_target(&mut self, target: PlayerId) -> bool {
        if target.is_none()
            || (!self.state_blob.is_empty()
                && self.state_blob.get(..4) != Some(Self::MBJINZU_HELPER_STATE_TAG.as_slice()))
        {
            return false;
        }
        let mut targets = self.mbjinzu_targets();
        if let Some((_, amount)) = targets.iter_mut().find(|(stored, _)| *stored == target) {
            *amount = amount.saturating_add(1);
        } else {
            targets.push((target, 1));
            targets.sort_by_key(|(target, _)| target.raw());
        }
        self.state_blob = Self::MBJINZU_HELPER_STATE_TAG.to_vec();
        for (target, amount) in targets {
            self.state_blob
                .extend_from_slice(&target.raw().to_le_bytes());
            self.state_blob.extend_from_slice(&amount.to_le_bytes());
        }
        true
    }

    pub(crate) fn take_mbjinzu_targets(&mut self) -> Vec<(PlayerId, u32)> {
        let targets = self.mbjinzu_targets();
        if !targets.is_empty() {
            self.state_blob.clear();
        }
        targets
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

    /// Physical cards tracked by this skill instance.
    pub(crate) fn tracked_cards(&self) -> Vec<CardId> {
        self.twkujian_cards()
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

    /// Add physical cards to this skill instance's stable tracked-card set.
    pub(crate) fn add_tracked_cards(&mut self, cards: &[CardId]) -> bool {
        let targets = self.active_play_phase_targets().unwrap_or_default();
        if !self.state_blob.is_empty()
            && self.state_blob.get(..4) != Some(Self::TWKUJIAN_STATE_TAG.as_slice())
            && self.state_blob.get(..4)
                != Some(Self::ACTIVE_PLAY_PHASE_TARGETS_STATE_TAG.as_slice())
            && self.state_blob.get(..4) != Some(Self::TRACKED_PLAY_TARGETS_STATE_TAG.as_slice())
        {
            return false;
        }
        let mut tracked = self.twkujian_cards();
        tracked.extend(cards.iter().copied().filter(|card| card.is_some()));
        tracked.sort_unstable_by_key(|card| card.raw());
        tracked.dedup();
        if targets.is_empty() {
            self.state_blob = Self::TWKUJIAN_STATE_TAG.to_vec();
        } else {
            self.state_blob = Self::TRACKED_PLAY_TARGETS_STATE_TAG.to_vec();
            self.state_blob
                .extend_from_slice(&(targets.len() as u32).to_le_bytes());
            for target in targets {
                self.state_blob
                    .extend_from_slice(&target.raw().to_le_bytes());
            }
        }
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

    /// Remove physical cards from the generic tracked-card set while preserving Play-phase
    /// target uniqueness state when both facets share the blob.
    pub(crate) fn take_tracked_cards(&mut self, cards: &[CardId]) -> Vec<CardId> {
        let targets = self.active_play_phase_targets().unwrap_or_default();
        let mut tracked = self.tracked_cards();
        let mut taken = Vec::new();
        tracked.retain(|card| {
            if cards.contains(card) {
                taken.push(*card);
                false
            } else {
                true
            }
        });
        if targets.is_empty() {
            self.state_blob = Self::TWKUJIAN_STATE_TAG.to_vec();
        } else if tracked.is_empty() {
            self.state_blob = Self::ACTIVE_PLAY_PHASE_TARGETS_STATE_TAG.to_vec();
            for target in targets {
                self.state_blob
                    .extend_from_slice(&target.raw().to_le_bytes());
            }
            return taken;
        } else {
            self.state_blob = Self::TRACKED_PLAY_TARGETS_STATE_TAG.to_vec();
            self.state_blob
                .extend_from_slice(&(targets.len() as u32).to_le_bytes());
            for target in targets {
                self.state_blob
                    .extend_from_slice(&target.raw().to_le_bytes());
            }
        }
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

    /// ABI 523 制霸：record the Wu-faction actor whose comparison this skill instance is
    /// pending on. The skill owner is the lord and the compare target.
    pub(crate) fn set_zhiba_compare(&mut self, actor: PlayerId) {
        self.state_blob = Self::ZHIBA_STATE_TAG.to_vec();
        self.state_blob
            .extend_from_slice(&actor.raw().to_le_bytes());
    }

    /// The Wu-faction actor of a pending 制霸 comparison, if any.
    pub(crate) fn zhiba_compare(&self) -> Option<PlayerId> {
        (self.state_blob.len() == 8
            && self.state_blob.get(..4) == Some(Self::ZHIBA_STATE_TAG.as_slice()))
        .then(|| {
            PlayerId::new(u32::from_le_bytes(
                self.state_blob[4..8].try_into().unwrap(),
            ))
        })
    }

    pub(crate) fn clear_zhiba_compare(&mut self) {
        if self.zhiba_compare().is_some() {
            self.state_blob.clear();
        }
    }

    pub(crate) const BOUND_TARGET_STATE_TAG: [u8; 4] = *b"BND1";

    pub(crate) const CARD_COLOR_LOCK_STATE_TAG: [u8; 4] = *b"CCL1";

    pub(crate) const CARD_USE_POLICY_STATE_TAG: [u8; 4] = *b"CUP1";

    pub(crate) const PHASE_CARD_USE_POLICY_STATE_TAG: [u8; 4] = *b"CUP2";

    pub(crate) const ACTIVE_UNIQUE_STATE_TAG: [u8; 4] = *b"AUC1";

    pub(crate) const ACTIVE_PLAY_PHASE_TARGETS_STATE_TAG: [u8; 4] = *b"APT1";

    pub(crate) const ACTIVE_COMPARE_HISTORY_STATE_TAG: [u8; 4] = *b"ACH1";

    pub(crate) const REMEMBERED_CATEGORY_STATE_TAG: [u8; 4] = *b"RCG1";

    pub(crate) const BOUND_CATEGORY_TARGET_STATE_TAG: [u8; 4] = *b"BCT1";

    pub(crate) const PHASE_GIFT_TARGET_STATE_TAG: [u8; 4] = *b"PGT1";

    pub(crate) const DELEGATED_RETRY_LOCK_STATE_TAG: [u8; 4] = *b"DRL1";

    pub(crate) const REMEMBERED_HAND_CARDS_STATE_TAG: [u8; 4] = *b"RHC1";

    pub(crate) const SOURCE_TURN_BOUND_STATE_TAG: [u8; 4] = *b"STB1";

    pub(crate) const NZRY_LONGNU_STATE_TAG: [u8; 4] = *b"LNU1";

    pub(crate) const DRLT_DUORUI_MAIN_STATE_TAG: [u8; 4] = *b"DRM1";

    pub(crate) const DRLT_DUORUI_HELPER_STATE_TAG: [u8; 4] = *b"DRH1";

    pub(crate) const LIFETIME_TARGETS_STATE_TAG: [u8; 4] = *b"LTT1";

    pub(crate) const TWLIHUO_DYING_COST_STATE_TAG: [u8; 4] = *b"TLC1";
}
