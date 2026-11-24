//! Track P3-E3 (`AUDIT_36` + `xinfu_zhanji`) real-event parity tests for all
//! remaining 37 audited skills: 3 baseline OK + 16 shared VM/kernel fixes +
//! 18 `HostBehavior::L11VmFixC` stripped skills.
//! Zero `debug_execute_effect` / `debug_resolve` calls; public-info AI checks
//! and NGSA save/restore roundtrips included.

#[path = "l11_npc_b_support/mod.rs"]
mod support;

#[path = "l11_vm_fix_c_parity_tests/mod.rs"]
mod l11_vm_fix_c_parity_tests;
