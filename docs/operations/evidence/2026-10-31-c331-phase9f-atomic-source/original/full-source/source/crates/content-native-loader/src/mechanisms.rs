//! Finite admission at the existing native bridge: data support is not runtime support.
use super::*;
use content_schema::{ContentPackage, HiddenTriggerPolicy, RevealBinding, TriggerRepeat};

pub(super) fn validate_runtime_mechanisms(pkg: &ContentPackage) -> Result<(), BridgeError> {
    if !pkg.has_native_mechanisms() {
        return Ok(());
    }
    for skill in &pkg.skill_index {
        if let Some(key) = &skill.onremove {
            if HostBehavior::from_key(key).is_none() {
                return Err(invalid(
                    "skill_index.onremove",
                    "unknown_native_cleanup_key",
                ));
            }
            // No existing factory exposes a cleanup-role invocation through this declaration.
            return Err(invalid("skill_index.onremove", "unsupported_cleanup_role"));
        }
        if skill.charlotte == Some(true)
            || skill.persevere == Some(true)
            || skill.forever == Some(true)
            || skill.temporary == Some(true)
        {
            return Err(invalid(
                "skill_index",
                "unsupported_skill_traits_or_temporary_lifetime",
            ));
        }
        if skill
            .group_children
            .as_ref()
            .is_some_and(|children| !children.is_empty())
        {
            return Err(invalid(
                "skill_index.group_children",
                "unsupported_provider_group_lifetime",
            ));
        }
        if skill
            .reveal
            .as_ref()
            .is_some_and(|reveal| *reveal != RevealBinding::None)
        {
            return Err(invalid("skill_index.reveal", "unsupported_reveal_binding"));
        }
    }
    for trigger in &pkg.trigger_index {
        if let Some(key) = &trigger.host_key {
            validate_native_key(key)?;
        }
        if trigger.force_die == Some(true)
            || trigger.force_out == Some(true)
            || trigger.repeat.is_some_and(|r| r != TriggerRepeat::Once)
            || trigger
                .hidden
                .is_some_and(|h| h != HiddenTriggerPolicy::PublicOnly)
            || trigger.event_skill.is_some()
        {
            return Err(invalid(
                "trigger_index",
                "unsupported_trigger_participation_or_named_window",
            ));
        }
    }
    for active in &pkg.active_index {
        if active.window == "dying" {
            return Err(invalid(
                "active_index.window",
                "unsupported_rescue_action_window",
            ));
        }
        if let Some(key) = &active.host_key {
            validate_native_key(key)?;
        }
    }
    Ok(())
}
fn validate_native_key(key: &str) -> Result<(), BridgeError> {
    if HostBehavior::from_key(key).is_none() {
        return Err(invalid("host_key", "unknown_native_execution_key"));
    }
    // A known ordinary body key is not proof that its explicit window/filter is implemented.
    Err(invalid(
        "host_key",
        "unsupported_explicit_native_declaration_binding",
    ))
}
