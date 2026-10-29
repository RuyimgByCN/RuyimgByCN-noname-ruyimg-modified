//! Deterministic native rule kernel and shared content vocabulary.
#![deny(unreachable_patterns)]

pub mod ids;
pub mod rng;
pub mod choice;
pub mod event;
pub mod catalog;
pub mod state;
pub mod machine;
pub mod resolution;
pub mod vocabulary;
pub mod kernel;
pub mod snap;
pub mod archive;
pub mod rvm_ops;
pub mod rvm_bridge;
pub mod rvm_named;

pub use machine::LogEvent;
