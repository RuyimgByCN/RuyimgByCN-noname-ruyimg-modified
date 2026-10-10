//! Content package schema, Rule IR, and versioned package format.
//!
//! Architecture §10.4–10.5.

#![deny(missing_docs)]

mod ir;
mod package;
mod sourcemap;

pub use ir::*;
pub use package::*;
pub use sourcemap::*;

pub use engine_api::{CONTENT_SCHEMA_VERSION, PACKAGE_FORMAT_VERSION};
