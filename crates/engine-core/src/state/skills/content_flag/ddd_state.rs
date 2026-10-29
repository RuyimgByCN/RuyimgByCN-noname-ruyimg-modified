//! DDD 技能实例的状态读写。

use super::*;

impl SkillInstance {
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
}
