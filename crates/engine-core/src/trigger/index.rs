//! Active trigger instance index — updated on skill add/remove only.

use super::catalog::TriggerDefId;
use super::{EventKind, TriggerOrderKey};
use crate::content::ContentRegistry;
use engine_api::{PlayerId, SkillDefId, SkillId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One active trigger instance in a match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveTrigger {
    /// Skill instance.
    pub skill_id: SkillId,
    /// Skill definition.
    pub skill_def: SkillDefId,
    /// Owner.
    pub owner: PlayerId,
    /// Trigger definition id.
    pub trigger_def: TriggerDefId,
    /// Event kind (denormalized for fast lookup).
    pub event: EventKind,
    /// Timing / priority / optional copied for sort without catalog fetch.
    pub timing: u16,
    /// Priority.
    pub priority: i32,
    /// Optional flag.
    pub optional: bool,
    /// Enabled (skill enabled).
    pub enabled: bool,
}

/// Match-scoped active trigger index.
///
/// Not part of canonical state hash — rebuilt from skills + catalog after
/// snapshot restore. Must stay consistent with skill store.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveTriggerIndex {
    /// All active rows (stable skill_id, trigger_def order maintained on rebuild).
    entries: Vec<ActiveTrigger>,
    /// EventKind.tag → indices into entries.
    by_event: BTreeMap<u32, Vec<usize>>,
}

impl ActiveTriggerIndex {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.by_event.clear();
    }

    /// Number of active rows.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Register all triggers for a skill instance (catalog from content registry).
    pub fn register_skill(
        &mut self,
        skill_id: SkillId,
        skill_def: SkillDefId,
        owner: PlayerId,
        content: &ContentRegistry,
    ) {
        for def in content.defs_for_skill(skill_def) {
            self.push_def(skill_id, skill_def, owner, &def, true);
        }
        self.reindex();
    }

    /// Unregister all triggers for a skill instance.
    pub fn unregister_skill(&mut self, skill_id: SkillId) {
        self.entries.retain(|e| e.skill_id != skill_id);
        self.reindex();
    }

    /// Set enabled flag for all rows of a skill instance.
    pub fn set_skill_enabled(&mut self, skill_id: SkillId, enabled: bool) {
        for e in &mut self.entries {
            if e.skill_id == skill_id {
                e.enabled = enabled;
            }
        }
    }

    /// Rebuild entirely from a list of (skill_id, skill_def, owner, enabled).
    pub fn rebuild_from<'a, I>(&mut self, skills: I, content: &ContentRegistry)
    where
        I: IntoIterator<Item = (SkillId, SkillDefId, PlayerId, bool)>,
    {
        self.clear();
        for (sid, sdef, owner, enabled) in skills {
            for def in content.defs_for_skill(sdef) {
                self.push_def(sid, sdef, owner, &def, enabled);
            }
        }
        self.reindex();
    }

    fn push_def(
        &mut self,
        skill_id: SkillId,
        skill_def: SkillDefId,
        owner: PlayerId,
        def: &crate::trigger::TriggerDefinition,
        enabled: bool,
    ) {
        self.entries.push(ActiveTrigger {
            skill_id,
            skill_def,
            owner,
            trigger_def: def.id,
            event: def.event,
            timing: def.timing as u16,
            priority: def.priority,
            optional: def.optional,
            enabled,
        });
    }

    fn reindex(&mut self) {
        // Stable order by skill_id then trigger_def for determinism of storage
        self.entries
            .sort_by_key(|e| (e.skill_id.raw(), e.trigger_def.raw()));
        self.by_event.clear();
        for (i, e) in self.entries.iter().enumerate() {
            self.by_event.entry(e.event.tag()).or_default().push(i);
        }
    }

    /// Collect matching enabled triggers for an event, sorted by total order.
    pub fn collect_sorted(
        &self,
        event: EventKind,
        order_keys: impl Fn(&ActiveTrigger) -> TriggerOrderKey,
    ) -> Vec<(TriggerOrderKey, &ActiveTrigger)> {
        let Some(idxs) = self.by_event.get(&event.tag()) else {
            return Vec::new();
        };
        let mut out: Vec<(TriggerOrderKey, &ActiveTrigger)> = idxs
            .iter()
            .filter_map(|&i| {
                let e = self.entries.get(i)?;
                if !e.enabled {
                    return None;
                }
                Some((order_keys(e), e))
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Iterate all entries (tests).
    pub fn iter(&self) -> impl Iterator<Item = &ActiveTrigger> {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trigger::catalog::BOOTSTRAP_JIANXIONG;
    use engine_api::SkillId;

    #[test]
    fn register_unregister_updates_index() {
        let mut idx = ActiveTriggerIndex::new();
        let sid = SkillId::new(1);
        let content = ContentRegistry::new();
        idx.register_skill(
            sid,
            SkillDefId::new(BOOTSTRAP_JIANXIONG),
            PlayerId::new(1),
            &content,
        );
        assert_eq!(idx.len(), 1);
        idx.unregister_skill(sid);
        assert!(idx.is_empty());
    }
}
