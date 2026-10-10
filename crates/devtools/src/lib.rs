//! Developer toolchain for noname-next (architecture plan §35).
//!
//! Implements the inspectors, profilers, and verifiers required by §35 as
//! testable library functions. Each module covers one tool:
//!
//! - [`timeline`] — Rule Timeline Inspector (§35)
//! - [`query_viewer`] — Query Explanation Viewer (§35)
//! - [`state_diff`] — State Diff Viewer (§35)
//! - [`bisector`] — State Hash Divergence Bisector (§35)
//! - [`minimizer`] — Replay Minimizer (§35)
//! - [`fuel_profiler`] — VM Fuel Profiler (§35)
//! - [`leak_harness`] — Observer Leak Test Harness (§35)
//! - [`package_runner`] — Content Package Test Runner (§35)

#![deny(missing_docs)]

pub mod bisector;
pub mod fuel_profiler;
pub mod leak_harness;
pub mod minimizer;
pub mod package_runner;
pub mod query_viewer;
pub mod state_diff;
pub mod timeline;
