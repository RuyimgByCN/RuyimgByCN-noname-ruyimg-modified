//! Engine / package compatibility checks.

use content_schema::PackageManifest;
use engine_api::{VersionBundle, RULE_VM_ABI_VERSION};

use crate::RuntimeError;

const OLDEST_ADDITIVE_RULE_VM_ABI: u32 = 3;
fn rule_vm_abi_compatible(host: u32, package: u32) -> bool {
    host == package
        || (host <= RULE_VM_ABI_VERSION && package >= OLDEST_ADDITIVE_RULE_VM_ABI && package < host)
}

/// Check package manifest against host engine versions.
pub fn check_engine_compat(
    manifest: &PackageManifest,
    host: &VersionBundle,
) -> Result<(), RuntimeError> {
    if host.engine_api < manifest.min_engine_api || host.engine_api > manifest.max_engine_api {
        return Err(RuntimeError::Incompatible(format!(
            "engine_api {} not in package range [{}, {}]",
            host.engine_api, manifest.min_engine_api, manifest.max_engine_api
        )));
    }
    if !rule_vm_abi_compatible(host.rule_vm_abi, manifest.rule_vm_abi_version) {
        return Err(RuntimeError::Incompatible(format!(
            "rule_vm_abi host={} package={}",
            host.rule_vm_abi, manifest.rule_vm_abi_version
        )));
    }
    if !matches!((host.content_schema, manifest.content_schema_version), (5, 3 | 4 | 5) | (4, 3 | 4) | (3, 3)) {
        return Err(RuntimeError::Incompatible(format!(
            "content_schema host={} package={}",
            host.content_schema, manifest.content_schema_version
        )));
    }
    if host.package_format != 0 && host.package_format != manifest.package_format_version {
        return Err(RuntimeError::Incompatible(format!(
            "package_format host={} package={}",
            host.package_format, manifest.package_format_version
        )));
    }
    Ok(())
}

/// Loose host bundle for Phase 10 tooling (matches content-schema defaults).
pub fn tooling_host_versions() -> VersionBundle {
    let mut v = VersionBundle::current();
    v.content_schema = content_schema::CONTENT_SCHEMA_VERSION;
    v.package_format = content_schema::PACKAGE_FORMAT_VERSION;
    v.engine_api = 1;
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn additive_hosts_accept_older_additive_packages() {
        let mut package = PackageManifest::new("test", "legacy", "1.0.0");
        package.rule_vm_abi_version = 3;

        check_engine_compat(&package, &VersionBundle::current())
            .expect("the current ABI is an additive superset of ABI 3");

        let mut host = VersionBundle::current();
        host.rule_vm_abi = 4;
        check_engine_compat(&package, &host).expect("ABI 4 is an additive superset of ABI 3");
    }

    #[test]
    fn abi3_host_rejects_abi4_package() {
        let mut package = PackageManifest::new("test", "current", "1.0.0");
        package.rule_vm_abi_version = 4;
        let mut host = VersionBundle::current();
        host.rule_vm_abi = 3;

        let error = check_engine_compat(&package, &host).expect_err("ABI 3 lacks ABI 4 effects");
        assert!(error.to_string().contains("rule_vm_abi host=3 package=4"));
    }

    #[test]
    fn abi4_host_rejects_abi5_package() {
        let mut package = PackageManifest::new("test", "current", "1.0.0");
        package.rule_vm_abi_version = 5;
        let mut host = VersionBundle::current();
        host.rule_vm_abi = 4;

        let error = check_engine_compat(&package, &host).expect_err("ABI 4 lacks card_equip");
        assert!(error.to_string().contains("rule_vm_abi host=4 package=5"));
    }

    #[test]
    fn abi5_host_rejects_abi6_package() {
        let mut package = PackageManifest::new("test", "current", "1.0.0");
        package.rule_vm_abi_version = 6;
        let mut host = VersionBundle::current();
        host.rule_vm_abi = 5;

        let error = check_engine_compat(&package, &host)
            .expect_err("ABI 5 lacks the alive_group_count host query");
        assert!(error.to_string().contains("rule_vm_abi host=5 package=6"));
    }

    #[test]
    fn abi6_host_rejects_abi7_package() {
        let mut package = PackageManifest::new("test", "current", "1.0.0");
        package.rule_vm_abi_version = 7;
        let mut host = VersionBundle::current();
        host.rule_vm_abi = 6;

        let error = check_engine_compat(&package, &host)
            .expect_err("ABI 6 lacks the unique_max_current_hp host query");
        assert!(error.to_string().contains("rule_vm_abi host=6 package=7"));
    }

    #[test]
    fn abi7_host_rejects_abi8_package() {
        let mut package = PackageManifest::new("test", "current", "1.0.0");
        package.rule_vm_abi_version = 8;
        let mut host = VersionBundle::current();
        host.rule_vm_abi = 7;

        let error = check_engine_compat(&package, &host)
            .expect_err("ABI 7 lacks the lifetime_damage_dealt host query");
        assert!(error.to_string().contains("rule_vm_abi host=7 package=8"));
    }

    #[test]
    fn abi8_host_rejects_abi9_package() {
        let mut package = PackageManifest::new("test", "current", "1.0.0");
        package.rule_vm_abi_version = 9;
        let mut host = VersionBundle::current();
        host.rule_vm_abi = 8;

        let error = check_engine_compat(&package, &host)
            .expect_err("ABI 8 lacks the adjust_hand_to host effect");
        assert!(error.to_string().contains("rule_vm_abi host=8 package=9"));
    }

    #[test]
    fn previous_host_rejects_current_package() {
        let package = PackageManifest::new("test", "current", "1.0.0");
        let mut host = VersionBundle::current();
        host.rule_vm_abi -= 1;

        let error = check_engine_compat(&package, &host)
            .expect_err("the previous ABI lacks the current host surface");
        assert!(error.to_string().contains(&format!(
            "rule_vm_abi host={} package={RULE_VM_ABI_VERSION}",
            RULE_VM_ABI_VERSION - 1
        )));
    }
}
