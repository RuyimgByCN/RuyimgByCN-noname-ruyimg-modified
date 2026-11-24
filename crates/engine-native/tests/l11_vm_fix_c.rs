//! Track P3-E3 (`FIX_27`) real-event integration tests for all 24 core
//! stripped `HostBehavior::L11VmFixC` skills (+ 6 subskills) and 3 canonical
//! cross-skills (`taoluan`, `xinfu_xionghuo`, `xinhongyan`).
//! Zero `debug_execute_effect` / `debug_resolve` calls; public-info AI checks
//! and NGSA save/restore roundtrips included.

#[path = "l11_npc_b_support/mod.rs"]
mod support;

#[path = "l11_vm_fix_c_tests/mod.rs"]
mod l11_vm_fix_c_tests;
