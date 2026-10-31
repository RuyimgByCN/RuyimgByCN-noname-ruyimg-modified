//! Trigger ordering is additive only for packages explicitly published as schema 4.
use super::*;

/// Static trigger timing band.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerTiming {
    /// Automatically resolve before normal owner buckets.
    First,
    /// Resolve in owner seat order, then descending priority.
    #[default]
    Normal,
    /// Automatically resolve after normal owner buckets.
    Last,
}
impl TriggerTiming {
    pub(super) fn is_normal(&self) -> bool { *self == Self::Normal }
    /// Canonical ordering tag, independent of serde representation.
    pub const fn tag(self) -> u8 {
        match self { Self::First => 0, Self::Normal => 1, Self::Last => 2 }
    }
}
pub(super) fn default_order_selectable() -> bool { true }
pub(super) fn is_true(value: &bool) -> bool { *value }
pub(super) fn is_zero_priority(value: &i32) -> bool { *value == 0 }

impl PackageTriggerEntry {
    /// Whether this handler needs the schema 4 ordering extension.
    pub fn has_nondefault_order(&self) -> bool {
        self.timing != TriggerTiming::Normal || self.priority != 0 || !self.order_selectable
    }
}
impl ContentPackage {
    /// Choose the publishing schema from actual metadata, without changing old hashes.
    pub fn required_content_schema(&self) -> u32 {
        if self.trigger_index.iter().any(PackageTriggerEntry::has_nondefault_order) { 4 } else { 3 }
    }
    /// Validate the execution schema even when the caller supplies a host with schema zero.
    pub fn validate_trigger_order_schema(&self) -> Result<(), String> {
        if self.manifest.package_format_version != PACKAGE_FORMAT_VERSION {
            return Err(format!("unsupported package_format {}", self.manifest.package_format_version));
        }
        let required = self.required_content_schema();
        if self.manifest.content_schema_version != required {
            return Err(format!("content_schema {} requires {required} for trigger ordering", self.manifest.content_schema_version));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for skill in &self.skill_index {
            if skill.name.is_empty() || !ids.insert(skill.skill_def_id) || !names.insert(&skill.name) {
                return Err("conflicting logical skill identity".into());
            }
        }
        let mut modules = std::collections::BTreeSet::new();
        if self.modules.iter().any(|m| !modules.insert(&m.name)) { return Err("duplicate module identity".into()); }
        for trigger in &self.trigger_index {
            if !ids.contains(&trigger.skill_def_id) || !modules.contains(&trigger.module) {
                return Err("trigger references unknown skill/module identity".into());
            }
        }
        Ok(())
    }
}
