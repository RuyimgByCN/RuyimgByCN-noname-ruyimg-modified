//! Preserve immutable schema3 character assets that referenced handler module names.
use super::*;

/// Resolve old schema3 module references to the one stable logical skill identity.
/// Schema4 authors must name the logical skill directly; ambiguous aliases reject the package.
pub fn bridge_generals_with_vm_package(
    json: &str,
    package: &content_schema::ContentPackage,
) -> Result<BridgedGenerals, BridgeError> {
    let decls = bridge_vm_skill_decls(package)?.decls;
    let known: BTreeSet<_> = decls.iter().map(|d| d.name.clone()).collect();
    ensure_symbol_capacity()?;
    let mut parsed = parse_json(json)?;
    ensure_host_api_compatible(&parsed)?;
    if package.manifest.content_schema_version == 3 {
        let mut aliases = std::collections::BTreeMap::new();
        let entries = package
            .trigger_index
            .iter()
            .map(|t| (t.module.as_str(), t.skill_def_id))
            .chain(
                package
                    .active_index
                    .iter()
                    .map(|a| (a.module.as_str(), a.skill_def_id)),
            );
        for (module, id) in entries {
            let skill = package
                .skill_index
                .iter()
                .find(|s| s.skill_def_id == id)
                .ok_or_else(|| invalid("skill_index", "missing logical skill"))?;
            if !known.contains(&skill.name) {
                continue;
            }
            if known.contains(module) && module != skill.name {
                return Err(invalid(
                    "characters",
                    "module alias conflicts with logical skill identity",
                ));
            }
            if aliases
                .insert(module, skill.name.as_str())
                .is_some_and(|old| old != skill.name)
            {
                return Err(invalid(
                    "characters",
                    "module aliases multiple logical skill identities",
                ));
            }
        }
        if let Some(generals) = parsed["generals"].as_array_mut() {
            for general in generals {
                if let Some(skills) = general["skills"].as_array_mut() {
                    for skill in skills {
                        if let Some(name) = skill["name"]
                            .as_str()
                            .and_then(|name| aliases.get(name))
                            .copied()
                        {
                            skill["name"] = Value::String(name.to_owned());
                        }
                    }
                }
            }
        }
    }
    bridge_generals_inner(&parsed, true, &known, true)
}
