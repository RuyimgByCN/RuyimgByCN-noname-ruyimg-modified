//! Deterministic native rule kernel and shared content vocabulary.
#![deny(unreachable_patterns)]

pub mod archive;
pub mod catalog;
pub mod choice;
pub mod event;
pub mod ids;
pub mod kernel;
pub mod machine;
pub mod resolution;
pub mod rng;
pub mod rvm_bridge;
pub mod rvm_named;
pub mod rvm_ops;
pub mod snap;
pub mod state;
pub mod vocabulary;

pub use machine::LogEvent;
