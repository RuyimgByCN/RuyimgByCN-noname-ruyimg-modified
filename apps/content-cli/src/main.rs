//! content-cli — package store management and diagnostic pack demo.

use clap::{Parser, Subcommand};
use content_runtime::{
    apply_updates, import_extensions, load_artifacts, plan_updates, save_artifacts, sign_package,
    ArtifactRecord, ChannelManifest, InstallOptions, PackageSignature, PackageStore,
};
use diagnostics::{DiagnosticBuilder, FaultCode};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "content-cli",
    about = "noname-next content install / revoke / diagnostics"
)]
struct Cli {
    /// Package store root (default: ./.npack-store).
    #[arg(long, global = true, default_value = ".npack-store")]
    store: PathBuf,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Install a content package JSON (+ optional signature).
    Install {
        /// package.json path.
        package: PathBuf,
        /// signature.json path.
        #[arg(long)]
        sig: Option<PathBuf>,
        /// Verify key (or env CONTENT_SIGN_KEY).
        #[arg(long)]
        key: Option<String>,
        /// Allow unsigned (dev only).
        #[arg(long)]
        allow_unsigned: bool,
        /// Do not activate after install.
        #[arg(long)]
        no_activate: bool,
    },
    /// Sign package bytes → signature JSON.
    Sign {
        package: PathBuf,
        #[arg(long)]
        key: Option<String>,
        #[arg(long, default_value = "dev")]
        key_id: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// List installed packages.
    List,
    /// Show active set and content lock.
    Active,
    /// Activate installed version.
    Activate {
        namespace: String,
        package_id: String,
        version: String,
    },
    /// Roll back active set one step.
    Rollback,
    /// Revoke content hash.
    RevokeHash {
        content_hash: String,
        #[arg(long, default_value = "manual")]
        reason: String,
        #[arg(long, default_value_t = true)]
        deactivate: bool,
    },
    /// Revoke package slot (all versions).
    RevokeSlot {
        namespace: String,
        package_id: String,
        #[arg(long, default_value = "manual")]
        reason: String,
        #[arg(long, default_value_t = true)]
        deactivate: bool,
    },
    /// GC staging leftovers after crash.
    GcStaging,
    /// Discover, install, and atomically activate drop-in extension directories.
    ImportExtensions {
        /// Parent directory containing one folder per extension.
        root: PathBuf,
        /// Verify key (or env CONTENT_SIGN_KEY).
        #[arg(long)]
        key: Option<String>,
        /// Allow unsigned local extensions.
        #[arg(long)]
        allow_unsigned: bool,
    },
    /// Record retained artifact.
    RetainArtifact {
        #[arg(long)]
        build_id: String,
        #[arg(long)]
        platform: String,
        #[arg(long)]
        golden_hash: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    /// List retained artifacts.
    Artifacts,
    /// Emit a sample redacted diagnostic pack (for tooling checks).
    DiagSample {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Compile DSL (.skill) source(s) into a content package JSON.
    Pack {
        /// Input .skill file(s); concatenated in order.
        inputs: Vec<PathBuf>,
        /// Output package.json path.
        #[arg(long)]
        out: PathBuf,
        /// Package id.
        #[arg(long, default_value = "standard")]
        package_id: String,
        /// Package namespace.
        #[arg(long, default_value = "official")]
        namespace: String,
        /// Package version.
        #[arg(long, default_value = "0.1.0")]
        version: String,
        /// Optional JSON character roster to embed in the package.
        #[arg(long)]
        characters: Option<PathBuf>,
        /// Optional JSON card-definition roster to embed in the package.
        #[arg(long)]
        cards: Option<PathBuf>,
        /// Optional JSON containing dependencies, translations, license, skin_index, and assets.
        #[arg(long)]
        metadata: Option<PathBuf>,
        /// Optional source asset directory; copied beside the package as assets/.
        #[arg(long)]
        assets: Option<PathBuf>,
        /// Also write signature (requires --key).
        #[arg(long)]
        sign_key: Option<String>,
        #[arg(long, default_value = "dev")]
        key_id: String,
    },
    /// Plan updates from a channel.json against the store.
    ChannelPlan {
        /// Path to channel.json.
        channel: PathBuf,
    },
    /// Apply planned channel updates (install + activate latest per slot).
    ChannelApply {
        /// Directory containing channel.json and package files.
        channel_root: PathBuf,
        /// channel.json relative to root (default channel.json).
        #[arg(long, default_value = "channel.json")]
        manifest: PathBuf,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        allow_unsigned: bool,
        /// Dry-run: print plan only.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    if let Err(e) = run(Cli::parse()) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let store = PackageStore::open(&cli.store)?;
    match cli.cmd {
        Cmd::Install {
            package,
            sig,
            key,
            allow_unsigned,
            no_activate,
        } => {
            let package = package.canonicalize()?;
            let bytes = std::fs::read(&package)?;
            let signature = match sig {
                Some(p) => {
                    let t = std::fs::read_to_string(p)?;
                    Some(serde_json::from_str::<PackageSignature>(&t)?)
                }
                None => None,
            };
            let key = key.or_else(|| std::env::var("CONTENT_SIGN_KEY").ok());
            let opts = InstallOptions {
                activate: !no_activate,
                allow_unsigned,
                verify_key: key,
                host: content_runtime::tooling_host_versions(),
            };
            let r = store.install_with_assets(
                &bytes,
                signature.as_ref(),
                &opts,
                package.parent().ok_or("package path has no parent")?,
            )?;
            println!(
                "installed {}/{}@{} signed={} path={}",
                r.meta.namespace, r.meta.package_id, r.meta.version, r.meta.signed, r.store_path
            );
            if let Some(lock) = r.content_lock {
                println!("content_lock={}", lock.lock_hash);
            }
        }
        Cmd::Sign {
            package,
            key,
            key_id,
            out,
        } => {
            let key = key
                .or_else(|| std::env::var("CONTENT_SIGN_KEY").ok())
                .ok_or("provide --key or CONTENT_SIGN_KEY")?;
            let bytes = std::fs::read(&package)?;
            let sig = sign_package(&bytes, &key, &key_id)?;
            let text = serde_json::to_string_pretty(&sig)?;
            if let Some(p) = out {
                std::fs::write(p, &text)?;
            } else {
                println!("{text}");
            }
        }
        Cmd::List => {
            for m in store.list_installed()? {
                println!(
                    "{}/{}@{} signed={} hash={}",
                    m.namespace, m.package_id, m.version, m.signed, m.content_hash
                );
            }
        }
        Cmd::Active => {
            let active = store.read_active()?;
            println!("active={:?}", active.packages);
            let lock = store.current_lock()?;
            println!("content_lock={}", lock.lock_hash);
            for p in lock.packages {
                println!(
                    "  {}/{}@{} logic={}",
                    p.namespace, p.package_id, p.version, p.logic_hash
                );
            }
        }
        Cmd::Activate {
            namespace,
            package_id,
            version,
        } => {
            store.activate_version(&namespace, &package_id, &version)?;
            println!("activated {namespace}/{package_id}@{version}");
            println!("content_lock={}", store.current_lock()?.lock_hash);
        }
        Cmd::Rollback => {
            let prev = store.rollback()?;
            println!("rolled back to {:?}", prev.packages);
            println!("content_lock={}", store.current_lock()?.lock_hash);
        }
        Cmd::RevokeHash {
            content_hash,
            reason,
            deactivate,
        } => {
            store.revoke_content_hash(&content_hash, &reason, deactivate)?;
            println!("revoked hash {content_hash}");
        }
        Cmd::RevokeSlot {
            namespace,
            package_id,
            reason,
            deactivate,
        } => {
            store.revoke_slot(&namespace, &package_id, &reason, deactivate)?;
            println!("revoked slot {namespace}/{package_id}");
        }
        Cmd::GcStaging => {
            let n = store.gc_staging()?;
            println!("removed {n} staging dir(s)");
        }
        Cmd::ImportExtensions {
            root,
            key,
            allow_unsigned,
        } => {
            let report = import_extensions(
                &store,
                &root,
                &InstallOptions {
                    activate: true,
                    allow_unsigned,
                    verify_key: key.or_else(|| std::env::var("CONTENT_SIGN_KEY").ok()),
                    host: content_runtime::tooling_host_versions(),
                },
            )?;
            for package in report.installed {
                println!(
                    "imported {}/{}@{}",
                    package.namespace, package.package_id, package.version
                );
            }
            println!("content_lock={}", report.content_lock.lock_hash);
        }
        Cmd::RetainArtifact {
            build_id,
            platform,
            golden_hash,
            notes,
        } => {
            let mut reg = load_artifacts(&store)?;
            let lock = store.current_lock().ok();
            reg.retain(ArtifactRecord {
                build_id,
                platform,
                golden_hash,
                content_lock_hash: lock.map(|l| l.lock_hash),
                retained: true,
                notes: notes.unwrap_or_default(),
            });
            save_artifacts(&store, &reg)?;
            println!("retained {} artifacts", reg.artifacts.len());
        }
        Cmd::Artifacts => {
            let reg = load_artifacts(&store)?;
            for a in reg.artifacts.values() {
                println!(
                    "{}@{} retained={} golden={:?} lock={:?}",
                    a.build_id, a.platform, a.retained, a.golden_hash, a.content_lock_hash
                );
            }
        }
        Cmd::DiagSample { out } => {
            let lock = store.current_lock().ok();
            let pack = DiagnosticBuilder::new()
                .fault(FaultCode::StateDivergence)
                .message("sample diagnostic — no hidden info")
                .match_id("sample-match")
                .first_divergent_revision(1)
                .current_revision(2)
                .content_lock_hash(lock.map(|l| l.lock_hash).unwrap_or_default())
                .build()?;
            pack.assert_no_hidden_leaks()?;
            let text = pack.to_json_pretty()?;
            if let Some(p) = out {
                std::fs::write(&p, &text)?;
                println!("wrote {}", p.display());
            } else {
                println!("{text}");
            }
        }
        Cmd::Pack {
            inputs,
            out,
            package_id,
            namespace,
            version,
            characters,
            cards,
            metadata,
            assets,
            sign_key,
            key_id,
        } => {
            if inputs.is_empty()
                && characters.is_none()
                && cards.is_none()
                && metadata.is_none()
                && assets.is_none()
            {
                return Err(
                    "pack requires a .skill input, --characters, --cards, --metadata, or --assets"
                        .into(),
                );
            }
            let mut source = String::new();
            for p in &inputs {
                source.push_str(&std::fs::read_to_string(p)?);
                source.push('\n');
            }
            let mut pkg = content_compiler::compile_source_named(&package_id, &source)?;
            pkg.manifest.namespace = namespace;
            pkg.manifest.version = version;
            if let Some(path) = characters {
                pkg.character_index = serde_json::from_slice(&std::fs::read(path)?)?;
            }
            if let Some(path) = cards {
                pkg.card_index = serde_json::from_slice(&std::fs::read(path)?)?;
            }
            if let Some(path) = metadata {
                let metadata: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
                if let Some(value) = metadata.get("dependencies") {
                    pkg.manifest.dependencies = serde_json::from_value(value.clone())?;
                }
                if let Some(value) = metadata.get("translations") {
                    pkg.translations = serde_json::from_value(value.clone())?;
                }
                if let Some(value) = metadata.get("license") {
                    pkg.manifest.license = serde_json::from_value(value.clone())?;
                }
                if let Some(value) = metadata.get("skin_index") {
                    pkg.skin_index = serde_json::from_value(value.clone())?;
                }
                if let Some(value) = metadata.get("asset_manifest") {
                    pkg.asset_manifest = Some(serde_json::from_value(value.clone())?);
                }
                if let Some(value) = metadata.get("asset_paths") {
                    pkg.asset_paths = serde_json::from_value(value.clone())?;
                }
            }
            if let Some(asset_root) = assets.as_deref() {
                if pkg.asset_manifest.is_some() || !pkg.asset_paths.is_empty() {
                    return Err(
                        "--assets conflicts with metadata asset_manifest/asset_paths".into(),
                    );
                }
                let compiled = resource_compiler::compile_asset_directory(&package_id, asset_root)?;
                pkg.asset_manifest = Some(compiled.manifest);
                pkg.asset_paths = compiled.asset_paths;
            }
            pkg.manifest.logic_hash = pkg.compute_logic_hash();
            pkg.manifest.asset_hash = pkg.compute_asset_hash();
            pkg.validate()
                .map_err(|error| format!("invalid package: {error}"))?;
            validate_official_license(&pkg)?;
            let bytes = pkg.to_json_bytes().map_err(|e| e.to_string())?;
            let output_root = out
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| std::path::Path::new("."));
            std::fs::create_dir_all(output_root)?;
            if let Some(asset_root) = assets.as_deref() {
                resource_compiler::copy_asset_directory(asset_root, &output_root.join("assets"))?;
            }
            std::fs::write(&out, &bytes)?;
            println!(
                "wrote {} logic_hash={}",
                out.display(),
                hex_bytes(&pkg.manifest.logic_hash)
            );
            if let Some(key) = sign_key.or_else(|| std::env::var("CONTENT_SIGN_KEY").ok()) {
                let sig = sign_package(&bytes, &key, &key_id)?;
                let sig_path = out.with_extension("sig.json");
                std::fs::write(&sig_path, serde_json::to_string_pretty(&sig)?)?;
                println!("wrote {}", sig_path.display());
            }
        }
        Cmd::ChannelPlan { channel } => {
            let ch = ChannelManifest::load(&channel)?;
            let plan = plan_updates(&store, &ch)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Cmd::ChannelApply {
            channel_root,
            manifest,
            key,
            allow_unsigned,
            dry_run,
        } => {
            let man_path = if manifest.is_absolute() {
                manifest
            } else {
                channel_root.join(manifest)
            };
            let ch = ChannelManifest::load(&man_path)?;
            let plan = plan_updates(&store, &ch)?;
            println!(
                "channel={} actions={} up_to_date={}",
                plan.channel_id,
                plan.actions.len(),
                plan.up_to_date.len()
            );
            for a in &plan.actions {
                println!(
                    "  {} {} -> {}",
                    a.slot,
                    a.from_version.as_deref().unwrap_or("-"),
                    a.to_version
                );
            }
            if dry_run {
                return Ok(());
            }
            let key = key.or_else(|| std::env::var("CONTENT_SIGN_KEY").ok());
            let opts = InstallOptions {
                activate: true,
                allow_unsigned,
                verify_key: key,
                host: content_runtime::tooling_host_versions(),
            };
            let installed = apply_updates(&store, &channel_root, &ch, &plan, &opts)?;
            for i in installed {
                println!("installed {i}");
            }
            println!("content_lock={}", store.current_lock()?.lock_hash);
        }
    }
    Ok(())
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn validate_official_license(package: &content_schema::ContentPackage) -> Result<(), String> {
    if package.manifest.namespace != "official" {
        return Ok(());
    }
    match licensing_schema::validate_for_official(&package.manifest.license) {
        licensing_schema::LicenseValidationResult::Ok => Ok(()),
        licensing_schema::LicenseValidationResult::Rejected(issues) => Err(format!(
            "official package licensing rejected: {}",
            issues.join("; ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_pack_requires_reviewed_license() {
        let mut package = content_compiler::compile_source_named("test", "").unwrap();
        package.manifest.namespace = "official".into();
        assert!(validate_official_license(&package).is_err());
        package.manifest.license.code_license = licensing_schema::LicenseType::Mit;
        package.manifest.license.default_asset_license = licensing_schema::LicenseType::Cc0;
        assert!(validate_official_license(&package).is_ok());
        package.manifest.namespace = "community".into();
        package.manifest.license = Default::default();
        assert!(validate_official_license(&package).is_ok());
    }
}
