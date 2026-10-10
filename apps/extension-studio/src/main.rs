//! extension-studio — headless migration + pack workflow (GUI later).

use clap::{Parser, Subcommand};
use content_runtime::{sign_package, InstallOptions, PackageStore};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "extension-studio",
    about = "Analyze legacy skills, emit skeletons, pack DSL, install to store"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Analyze legacy skill.js → report + optional skeletons.
    Analyze {
        input: PathBuf,
        #[arg(long)]
        md: Option<PathBuf>,
        #[arg(long)]
        json: Option<PathBuf>,
        #[arg(long)]
        skeletons: Option<PathBuf>,
    },
    /// Pack one or more .skill files into package.json (+ optional sign/install).
    Publish {
        /// .skill sources.
        inputs: Vec<PathBuf>,
        #[arg(long, default_value = "standard")]
        package_id: String,
        /// Package namespace.
        #[arg(long, default_value = "community")]
        namespace: String,
        /// Package version.
        #[arg(long, default_value = "0.1.0")]
        version: String,
        /// Optional JSON containing dependencies, translations, license, skin_index, and assets.
        #[arg(long)]
        metadata: Option<PathBuf>,
        /// Optional source asset directory; copied beside the package as assets/.
        #[arg(long)]
        assets: Option<PathBuf>,
        /// Write package.json here.
        #[arg(long)]
        out: PathBuf,
        /// Sign with key (or CONTENT_SIGN_KEY).
        #[arg(long)]
        key: Option<String>,
        #[arg(long, default_value = "dev")]
        key_id: String,
        /// Install into store after pack.
        #[arg(long)]
        store: Option<PathBuf>,
        #[arg(long)]
        allow_unsigned: bool,
    },
    /// Grade-only summary for a legacy file (stdout).
    Grade { input: PathBuf },
}

fn main() {
    if let Err(e) = run(Cli::parse()) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.cmd {
        Cmd::Analyze {
            input,
            md,
            json,
            skeletons,
        } => {
            let report = legacy_analyzer::analyze_file(&input)?;
            println!(
                "skills={} counts={:?} — {}",
                report.skills.len(),
                report.counts,
                report.disclaimer
            );
            for s in &report.skills {
                println!("  [{}] {}", s.grade.as_str(), s.name);
            }
            if let Some(p) = json {
                std::fs::write(p, report.to_json_pretty()?)?;
            }
            if let Some(p) = md {
                std::fs::write(p, report.to_markdown())?;
            }
            if let Some(dir) = skeletons {
                std::fs::create_dir_all(&dir)?;
                for s in &report.skills {
                    if s.skeleton_useful {
                        std::fs::write(dir.join(format!("{}.skill", s.name)), &s.dsl_skeleton)?;
                    }
                }
            }
        }
        Cmd::Publish {
            inputs,
            package_id,
            namespace,
            version,
            metadata,
            assets,
            out,
            key,
            key_id,
            store,
            allow_unsigned,
        } => {
            if inputs.is_empty() && metadata.is_none() && assets.is_none() {
                return Err("publish requires .skill inputs, --metadata, or --assets".into());
            }
            let mut source = String::new();
            for p in &inputs {
                source.push_str(&std::fs::read_to_string(p)?);
                source.push('\n');
            }
            let mut pkg = content_compiler::compile_source_named(&package_id, &source)?;
            pkg.manifest.namespace = namespace;
            pkg.manifest.version = version;
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
            println!("packed {} bytes → {}", bytes.len(), out.display());

            let key = key.or_else(|| std::env::var("CONTENT_SIGN_KEY").ok());
            let sig = if let Some(ref k) = key {
                let s = sign_package(&bytes, k, &key_id)?;
                let sig_path = out.with_extension("sig.json");
                std::fs::write(&sig_path, serde_json::to_string_pretty(&s)?)?;
                println!("signed → {}", sig_path.display());
                Some(s)
            } else {
                None
            };

            if let Some(store_path) = store {
                let st = PackageStore::open(store_path)?;
                let opts = InstallOptions {
                    activate: true,
                    allow_unsigned: allow_unsigned || sig.is_none(),
                    verify_key: key,
                    host: content_runtime::tooling_host_versions(),
                };
                if !opts.allow_unsigned && sig.is_none() {
                    return Err("install requires --key or --allow-unsigned".into());
                }
                let r = st.install_with_assets(&bytes, sig.as_ref(), &opts, output_root)?;
                println!(
                    "installed {}/{}@{} lock={}",
                    r.meta.namespace,
                    r.meta.package_id,
                    r.meta.version,
                    r.content_lock
                        .as_ref()
                        .map(|l| l.lock_hash.as_str())
                        .unwrap_or("-")
                );
            }
        }
        Cmd::Grade { input } => {
            let report = legacy_analyzer::analyze_file(&input)?;
            for s in report.skills {
                println!("{}\t{}", s.grade.as_str(), s.name);
            }
        }
    }
    Ok(())
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
