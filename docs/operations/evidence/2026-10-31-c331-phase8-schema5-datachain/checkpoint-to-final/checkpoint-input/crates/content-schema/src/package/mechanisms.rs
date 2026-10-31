//! Schema5 declaration data; runtime capability admission stays in the native loader.
use super::*;

/// Number of invocations sampled by a future indexed trigger consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerRepeat {
    /// One ordinary invocation.
    Once,
    /// One invocation per damage point.
    DamageAmount,
    /// Damage points, or one invocation for a non-damage event.
    DamageAmountElseOne,
}
/// Admission of concealed skill ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HiddenTriggerPolicy {
    /// Existing public ownership only.
    PublicOnly,
    /// Require a genuine reveal binding.
    AskReveal,
    /// Preserve concealed ownership.
    RunHidden,
}
/// Declared reveal action; handler references are genuine logical definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RevealBinding {
    /// No reveal mechanism.
    None,
    /// Genuine handlers around the public confirmation.
    TriggerHook {
        /// Handler before confirmation.
        before: Option<u32>,
        /// Handler after confirmation.
        after: Option<u32>,
    },
    /// Explicit public confirmation.
    ConfirmPublic,
}

pub(super) fn validate_wire_presence(value: &serde_json::Value) -> Result<(), String> {
    let schema = value["manifest"]["content_schema_version"].as_u64();
    if !matches!(schema, Some(3..=5)) {
        return Err("unsupported content_schema; expected3/4/5".into());
    }
    let fields: &[(&str, &[&str])] = &[
        ("skill_index", &["charlotte", "persevere", "forever", "temporary", "group_children", "reveal", "onremove"]),
        ("trigger_index", &["host_key", "force_die", "force_out", "repeat", "hidden", "event_skill"]),
        ("active_index", &["host_key"]),
    ];
    for (table, names) in fields {
        if let Some(entries) = value[*table].as_array() {
            for entry in entries {
                for key in *names {
                    if let Some(field) = entry.get(*key) {
                        if schema != Some(5) || field.is_null() {
                            return Err(format!("schema5 field {table}.{key} requires schema5 and a literal value"));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

impl ContentPackage {
    /// Presence, including explicit default values, requires schema5.
    pub fn has_native_mechanisms(&self) -> bool {
        self.skill_index.iter().any(|s| s.charlotte.is_some() || s.persevere.is_some()
            || s.forever.is_some() || s.temporary.is_some() || s.group_children.is_some()
            || s.reveal.is_some() || s.onremove.is_some())
            || self.trigger_index.iter().any(|t| t.host_key.is_some() || t.force_die.is_some()
                || t.force_out.is_some() || t.repeat.is_some() || t.hidden.is_some() || t.event_skill.is_some())
            || self.active_index.iter().any(|a| a.host_key.is_some())
    }

    /// Canonical declaration digest; preserves presence, positions and exact bindings.
    pub fn native_mechanisms_hash(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"npack-native-mechanisms/1");
        let mut segment = |data: Vec<u8>| { h.update(&(data.len() as u64).to_le_bytes()); h.update(&data); };
        for (position, s) in self.skill_index.iter().enumerate() {
            segment(serde_json::to_vec(&(position, s.skill_def_id, &s.name, s.locked, s.charlotte,
                s.persevere, s.forever, s.temporary, &s.group_children, &s.reveal, &s.onremove)).expect("typed declaration"));
        }
        for (position, t) in self.trigger_index.iter().enumerate() {
            segment(serde_json::to_vec(&(position, t)).expect("typed handler"));
        }
        for (position, a) in self.active_index.iter().enumerate() {
            segment(serde_json::to_vec(&(position, a)).expect("typed active handler"));
        }
        *h.finalize().as_bytes()
    }

    pub(super) fn validate_native_mechanisms(&self) -> Result<(), String> {
        if !self.has_native_mechanisms() { return Ok(()); }
        if self.skill_index.len() > u16::MAX as usize { return Err("skill definition capacity exceeded".into()); }
        let mut triggers = std::collections::BTreeSet::new();
        let mut actives = std::collections::BTreeSet::new();
        for t in &self.trigger_index {
            if !triggers.insert(t.trigger_def_id) { return Err("duplicate genuine handler identity".into()); }
            validate_executor(&t.module, &t.host_key, t.entry_pc)?;
            let boundary = matches!(t.event.as_str(), "skill_before" | "skill_begin" | "skill_end" | "skill_after");
            if boundary != t.event_skill.is_some() { return Err("event_skill requires a genuine named skill window".into()); }
            if matches!(t.repeat, Some(TriggerRepeat::DamageAmount)) && !matches!(t.event.as_str(), "damage" | "damage_before" | "damage_begin" | "damage_end" | "damage_after" | "damage_source") {
                return Err("damage_amount repeat requires a damage window".into());
            }
            if matches!(t.hidden, Some(HiddenTriggerPolicy::AskReveal)) {
                let skill = self.skill_index.iter().find(|s| s.skill_def_id == t.skill_def_id);
                if !skill.is_some_and(|s| matches!(s.reveal, Some(RevealBinding::ConfirmPublic | RevealBinding::TriggerHook { .. }))) {
                    return Err("ask_reveal requires a genuine reveal binding".into());
                }
            }
        }
        for a in &self.active_index {
            validate_executor(&a.module, &a.host_key, a.entry_pc)?;
            if !actives.insert(a.skill_def_id) { return Err("duplicate active handler identity".into()); }
            if !self.skill_index.iter().any(|s| s.skill_def_id == a.skill_def_id) { return Err("active references unknown logical skill".into()); }
            if self.has_native_mechanisms() && !matches!(a.window.as_str(), "play_phase" | "any" | "dying") { return Err("unknown active window".into()); }
        }
        for skill in &self.skill_index {
            if let Some(key) = &skill.onremove {
                if !key.starts_with("host:") || key.len() == 5 { return Err("invalid finite cleanup key".into()); }
            }
            let children = skill.group_children.as_deref().unwrap_or_default();
            let unique: std::collections::BTreeSet<_> = children.iter().collect();
            if children.len() > u16::MAX as usize || unique.len() != children.len() || children.contains(&skill.skill_def_id) { return Err("invalid group_children edges".into()); }
            if matches!(&skill.reveal, Some(RevealBinding::TriggerHook { before: None, after: None })) { return Err("empty reveal hook binding".into()); }
        }
        if self.manifest.dependencies.is_empty() { Self::validate_native_mechanism_graph(std::slice::from_ref(self))?; }
        Ok(())
    }

    /// Resolve numeric declaration edges only within the exact dependency-ordered package set.
    pub fn validate_native_mechanism_graph(packages: &[Self]) -> Result<(), String> {
        if !packages.iter().any(Self::has_native_mechanisms) { return Ok(()); }
        let mut graph = BTreeMap::<(usize,u32),Vec<(usize,u32)>>::new();
        for (index, package) in packages.iter().enumerate() {
            let mut allowed = std::collections::BTreeSet::from([index]);
            let mut pending = vec![index];
            while let Some(current) = pending.pop() {
                for d in &packages[current].manifest.dependencies {
                    let found = packages.iter().position(|p| p.manifest.namespace == d.namespace && p.manifest.package_id == d.package_id && p.manifest.version == d.version && (d.logic_hash == [0;32] || d.logic_hash == p.manifest.logic_hash))
                        .ok_or_else(|| "missing locked mechanism dependency".to_string())?;
                    if allowed.insert(found) { pending.push(found); }
                }
            }
            let resolve = |id: u32, handler: bool| -> Result<(usize,u32),String> {
                let candidates: Vec<_> = allowed.iter().copied().filter(|i| packages[*i].skill_index.iter().any(|s| s.skill_def_id == id)).collect();
                if candidates.len() != 1 { return Err(format!("unresolved or ambiguous genuine skill {id}")); }
                let i = candidates[0];
                if handler && !packages[i].trigger_index.iter().any(|t| t.skill_def_id == id) && !packages[i].active_index.iter().any(|a| a.skill_def_id == id) { return Err("reference is not a genuine handler".into()); }
                Ok((i,id))
            };
            for s in &package.skill_index {
                let mut children = Vec::new();
                for child in s.group_children.as_deref().unwrap_or_default() { children.push(resolve(*child, false)?); }
                graph.insert((index,s.skill_def_id), children);
                if let Some(RevealBinding::TriggerHook { before, after }) = &s.reveal {
                    for id in before.iter().chain(after.iter()) { resolve(*id,true)?; }
                }
            }
            for t in &package.trigger_index { if let Some(id) = t.event_skill { resolve(id,true)?; } }
        }
        let mut incoming: BTreeMap<_,usize> = graph.keys().map(|k| (*k,0)).collect();
        for children in graph.values() { for child in children { *incoming.get_mut(child).ok_or("dangling group child")? += 1; } }
        let mut ready: Vec<_> = incoming.iter().filter_map(|(k,n)| (*n == 0).then_some(*k)).collect();
        let mut count = 0;
        while let Some(key) = ready.pop() {
            count += 1;
            for child in &graph[&key] { let n=incoming.get_mut(child).expect("checked edge"); *n -= 1; if *n == 0 { ready.push(*child); } }
        }
        if count != graph.len() { return Err("cyclic group_children".into()); }
        Ok(())
    }
}
fn validate_executor(module: &str, host: &Option<String>, pc: u32) -> Result<(), String> {
    match host {
        Some(key) if module.is_empty() && pc == 0 && key.starts_with("host:") && key.len() > 5 => Ok(()),
        None if !module.is_empty() => Ok(()),
        _ => Err("exactly one verified module or native host_key is required; host entry_pc must be zero".into()),
    }
}

// Reject explicit null without losing Option presence in direct serde SDK callers.
pub(super) fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where D: serde::Deserializer<'de>, T: Deserialize<'de> {
    T::deserialize(deserializer).map(Some)
}
