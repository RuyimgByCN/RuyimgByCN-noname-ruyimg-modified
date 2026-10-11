//! CLI: legacy-analyze

use clap::{Parser, Subcommand};
use legacy_analyzer::{
    analyze_file, analyze_source, build_capability_matrix, generate_dsl_skeleton,
    sign_package_json, verify_package_signature, PackageSignature,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "legacy-analyze",
    about = "Noname legacy skill analyzer / migration aid"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Analyze a skill.js (or similar) file; write JSON/Markdown report.
    Analyze {
        /// Input path.
        input: PathBuf,
        /// Write JSON report here.
        #[arg(long)]
        json: Option<PathBuf>,
        /// Write Markdown report here.
        #[arg(long)]
        md: Option<PathBuf>,
        /// Directory to write per-skill `.skill` skeletons.
        #[arg(long)]
        skeletons: Option<PathBuf>,
    },
    /// Build the M0 source-backed capability matrix from labeled skill files.
    Matrix {
        /// Inputs as `label=path` (repeat at least twice for cross-pack coverage).
        #[arg(required = true)]
        inputs: Vec<String>,
        /// Complete machine-readable matrix.
        #[arg(long)]
        json: PathBuf,
        /// Human summary.
        #[arg(long)]
        md: PathBuf,
    },
    /// Count `const card = {}` segment of a shell extension card.js.
    CardStats {
        /// Input card.js path.
        input: PathBuf,
        /// Write JSON stats here.
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Emit skeletons only from stdin-less file.
    Skeleton {
        /// Input path.
        input: PathBuf,
        /// Output directory.
        #[arg(long, default_value = "out/skeletons")]
        out: PathBuf,
    },
    /// Hash file (blake3 hex).
    Hash {
        /// Input path.
        input: PathBuf,
    },
    /// Sign package bytes with keyed blake3.
    Sign {
        /// Package file.
        input: PathBuf,
        /// Key material (env LEGACY_SIGN_KEY overrides).
        #[arg(long)]
        key: Option<String>,
        /// Key id label.
        #[arg(long, default_value = "dev")]
        key_id: String,
        /// Write signature JSON.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Verify package signature JSON.
    Verify {
        /// Package file.
        input: PathBuf,
        /// Signature JSON file.
        sig: PathBuf,
        /// Key material.
        #[arg(long)]
        key: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.cmd {
        Cmd::Analyze {
            input,
            json,
            md,
            skeletons,
        } => {
            let report = analyze_file(&input)?;
            println!(
                "source={} skills={} counts={:?}",
                report.source,
                report.skills.len(),
                report.counts
            );
            println!("{}", report.disclaimer);
            if let Some(p) = json {
                std::fs::write(&p, report.to_json_pretty()?)?;
                println!("wrote {}", p.display());
            }
            if let Some(p) = md {
                std::fs::write(&p, report.to_markdown())?;
                println!("wrote {}", p.display());
            }
            if let Some(dir) = skeletons {
                std::fs::create_dir_all(&dir)?;
                for s in &report.skills {
                    if !s.skeleton_useful {
                        continue;
                    }
                    let path = dir.join(format!("{}.skill", s.name));
                    std::fs::write(&path, &s.dsl_skeleton)?;
                }
                println!("skeletons → {}", dir.display());
            }
            // brief stdout listing
            for s in &report.skills {
                println!(
                    "  [{}] {} — {}",
                    s.grade.as_str(),
                    s.name,
                    s.reasons.first().cloned().unwrap_or_default()
                );
            }
        }
        Cmd::Matrix { inputs, json, md } => {
            let mut sources = Vec::new();
            for input in inputs {
                let (label, path) = input
                    .split_once('=')
                    .ok_or("matrix input must be label=path")?;
                let source = std::fs::read_to_string(path)?;
                sources.push((label.to_string(), analyze_source(&source)?));
            }
            let matrix = build_capability_matrix(sources);
            std::fs::write(&json, matrix.to_json_pretty()?)?;
            std::fs::write(&md, matrix.to_markdown())?;
            println!(
                "wrote {} skills to {} and {}",
                matrix.total_skills,
                json.display(),
                md.display()
            );
        }
        Cmd::CardStats { input, json } => {
            let source = std::fs::read_to_string(&input)?;
            let stats = legacy_analyzer::count_card_segment(&source);
            if let Some(path) = json {
                std::fs::write(&path, serde_json::to_string_pretty(&stats)?)?;
                println!("written {}", path.display());
            }
            println!(
                "const card 段：共 {} 张（纯数据 {} / 含行为 {}）",
                stats.total, stats.data_only, stats.with_behavior
            );
            for (t, n) in &stats.type_counts {
                println!("  type={t}: {n}");
            }
            for e in &stats.entries {
                println!(
                    "  {} [{}]{}{}",
                    e.name,
                    e.card_type.as_deref().unwrap_or("-"),
                    if e.fullskin { " fullskin" } else { "" },
                    if e.has_behavior { " behavior" } else { "" },
                );
            }
        }
        Cmd::Skeleton { input, out } => {
            let source = std::fs::read_to_string(&input)?;
            let profiles = analyze_source(&source)?;
            std::fs::create_dir_all(&out)?;
            for (i, p) in profiles.iter().enumerate() {
                let sk = generate_dsl_skeleton(p, (i as u32) + 1);
                let path = out.join(format!("{}.skill", p.name));
                std::fs::write(&path, sk)?;
            }
            println!("wrote {} skeletons to {}", profiles.len(), out.display());
        }
        Cmd::Hash { input } => {
            let data = std::fs::read(&input)?;
            println!("{}", legacy_analyzer::logic_hash_bytes(&data));
        }
        Cmd::Sign {
            input,
            key,
            key_id,
            out,
        } => {
            let key = key
                .or_else(|| std::env::var("LEGACY_SIGN_KEY").ok())
                .ok_or("provide --key or LEGACY_SIGN_KEY")?;
            let data = std::fs::read(&input)?;
            let sig = sign_package_json(&data, &key, &key_id)?;
            let text = serde_json::to_string_pretty(&sig)?;
            if let Some(p) = out {
                std::fs::write(&p, &text)?;
                println!("wrote {}", p.display());
            } else {
                println!("{text}");
            }
        }
        Cmd::Verify { input, sig, key } => {
            let key = key
                .or_else(|| std::env::var("LEGACY_SIGN_KEY").ok())
                .ok_or("provide --key or LEGACY_SIGN_KEY")?;
            let data = std::fs::read(&input)?;
            let sig_text = std::fs::read_to_string(&sig)?;
            let sig: PackageSignature = serde_json::from_str(&sig_text)?;
            let ok = verify_package_signature(&data, &key, &sig)?;
            if ok {
                println!("OK");
            } else {
                eprintln!("FAIL");
                std::process::exit(2);
            }
        }
    }
    Ok(())
}
