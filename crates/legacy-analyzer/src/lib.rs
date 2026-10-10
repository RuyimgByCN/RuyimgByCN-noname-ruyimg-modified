//! Legacy noname JavaScript skill analyzer and migration helpers.
//!
//! Architecture §17: extract structure, grade compatibility, emit DSL skeletons.
//! Does **not** claim semantic equivalence for arbitrary JS.

#![deny(missing_docs)]

mod extract;
mod grade;
mod matrix;
mod report;
mod sign;
mod skeleton;

pub use extract::{analyze_source, SkillProfile};
pub use grade::{grade_skill, CompatGrade, GradeDetail};
pub use matrix::{build_capability_matrix, CapabilityMatrix, SkillCapabilityRow};
pub use report::{MigrationReport, SkillReport};
pub use sign::{logic_hash_bytes, sign_package_json, verify_package_signature, PackageSignature};
pub use skeleton::generate_dsl_skeleton;

use thiserror::Error;

/// Analyzer / migration error.
#[derive(Debug, Error)]
pub enum AnalyzeError {
    /// I/O.
    #[error("io: {0}")]
    Io(String),
    /// Parse / extract failure.
    #[error("extract: {0}")]
    Extract(String),
    /// Invalid argument.
    #[error("invalid: {0}")]
    Invalid(String),
}

impl From<std::io::Error> for AnalyzeError {
    fn from(e: std::io::Error) -> Self {
        AnalyzeError::Io(e.to_string())
    }
}

/// Analyze a file path and build a full migration report.
pub fn analyze_file(path: &std::path::Path) -> Result<MigrationReport, AnalyzeError> {
    let source = std::fs::read_to_string(path)?;
    let profiles = analyze_source(&source)?;
    Ok(MigrationReport::from_profiles(
        path.display().to_string(),
        profiles,
    ))
}
