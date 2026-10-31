//! 已安装包的依赖排序：拓扑序、成环检测与槽位可达性。

use super::*;

pub(super) fn order_packages(
    packages: Vec<ContentPackage>,
) -> Result<Vec<ContentPackage>, RuntimeError> {
    let package_count = packages.len();
    let mut remaining: BTreeMap<String, ContentPackage> = packages
        .into_iter()
        .map(|package| {
            (
                ActiveSet::slot_key(&package.manifest.namespace, &package.manifest.package_id),
                package,
            )
        })
        .collect();
    if remaining.len() != package_count {
        return Err(RuntimeError::Invalid(
            "multiple active versions for one package slot".into(),
        ));
    }
    let all: BTreeMap<String, (String, [u8; 32])> = remaining
        .iter()
        .map(|(slot, package)| {
            (
                slot.clone(),
                (
                    package.manifest.version.clone(),
                    package.manifest.logic_hash,
                ),
            )
        })
        .collect();
    for package in remaining.values() {
        for dependency in &package.manifest.dependencies {
            let slot = ActiveSet::slot_key(&dependency.namespace, &dependency.package_id);
            let Some((version, logic_hash)) = all.get(&slot) else {
                return Err(RuntimeError::Invalid(format!(
                    "missing dependency {slot}@{} for {}/{}",
                    dependency.version, package.manifest.namespace, package.manifest.package_id
                )));
            };
            if version != &dependency.version
                || (dependency.logic_hash != [0u8; 32] && logic_hash != &dependency.logic_hash)
            {
                return Err(RuntimeError::Invalid(format!(
                    "dependency mismatch {slot}@{} for {}/{}",
                    dependency.version, package.manifest.namespace, package.manifest.package_id
                )));
            }
        }
    }

    let mut loaded = BTreeSet::new();
    let mut ordered = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let ready = remaining.iter().find_map(|(slot, package)| {
            package
                .manifest
                .dependencies
                .iter()
                .all(|dependency| {
                    loaded.contains(&ActiveSet::slot_key(
                        &dependency.namespace,
                        &dependency.package_id,
                    ))
                })
                .then(|| slot.clone())
        });
        if let Some(slot) = ready {
            loaded.insert(slot.clone());
            ordered.push(remaining.remove(&slot).expect("ready package exists"));
            continue;
        }

        let Some(component) = ready_cycle_component(&remaining) else {
            return Err(RuntimeError::Invalid(
                "unresolvable package dependency graph".into(),
            ));
        };
        for slot in component {
            loaded.insert(slot.clone());
            ordered.push(
                remaining
                    .remove(&slot)
                    .expect("cycle component package exists"),
            );
        }
    }
    ContentPackage::validate_native_mechanism_graph(&ordered).map_err(RuntimeError::Package)?;
    Ok(ordered)
}

pub(super) fn ready_cycle_component(
    remaining: &BTreeMap<String, ContentPackage>,
) -> Option<Vec<String>> {
    let edges: BTreeMap<String, BTreeSet<String>> = remaining
        .iter()
        .map(|(slot, package)| {
            let dependencies = package
                .manifest
                .dependencies
                .iter()
                .map(|dependency| {
                    ActiveSet::slot_key(&dependency.namespace, &dependency.package_id)
                })
                .filter(|dependency| remaining.contains_key(dependency))
                .collect();
            (slot.clone(), dependencies)
        })
        .collect();
    let mut reverse: BTreeMap<String, BTreeSet<String>> = remaining
        .keys()
        .map(|slot| (slot.clone(), BTreeSet::new()))
        .collect();
    for (slot, dependencies) in &edges {
        for dependency in dependencies {
            reverse
                .get_mut(dependency)
                .expect("remaining dependency has reverse entry")
                .insert(slot.clone());
        }
    }

    remaining.keys().find_map(|slot| {
        let forward = reachable_slots(slot, &edges);
        let backward = reachable_slots(slot, &reverse);
        let component: BTreeSet<String> = forward.intersection(&backward).cloned().collect();
        let cyclic = component.len() > 1
            || edges
                .get(slot)
                .is_some_and(|dependencies| dependencies.contains(slot));
        let dependencies_ready = component.iter().all(|member| {
            edges
                .get(member)
                .is_none_or(|dependencies| dependencies.is_subset(&component))
        });
        (cyclic && dependencies_ready).then(|| component.into_iter().collect())
    })
}

pub(super) fn reachable_slots(
    start: &str,
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut reached = BTreeSet::new();
    let mut pending = vec![start.to_owned()];
    while let Some(slot) = pending.pop() {
        if !reached.insert(slot.clone()) {
            continue;
        }
        if let Some(next) = edges.get(&slot) {
            pending.extend(next.iter().cloned());
        }
    }
    reached
}
