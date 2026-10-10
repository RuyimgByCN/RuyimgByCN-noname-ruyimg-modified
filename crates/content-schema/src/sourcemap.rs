//! Source map: bytecode PC → source span.

use crate::ir::Span;
use serde::{Deserialize, Serialize};

/// One mapping entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMapEntry {
    /// Bytecode PC (start of instruction).
    pub pc: u32,
    /// Source span.
    pub span: Span,
    /// Optional label.
    pub label: String,
}

/// Source map for a module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceMap {
    /// Module name.
    pub module: String,
    /// Entries sorted by pc.
    pub entries: Vec<SourceMapEntry>,
}

impl SourceMap {
    /// Lookup nearest entry at or before pc.
    pub fn lookup(&self, pc: u32) -> Option<&SourceMapEntry> {
        let mut best = None;
        for e in &self.entries {
            if e.pc <= pc {
                best = Some(e);
            } else {
                break;
            }
        }
        best
    }
}
