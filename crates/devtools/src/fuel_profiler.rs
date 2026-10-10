//! VM Fuel Profiler (architecture §35).
//!
//! Reports fuel consumption per bytecode module execution. Helps identify
//! content packages whose skills burn excessive fuel.

use rule_vm::bytecode::BytecodeModule;
use rule_vm::{CollectingHost, ExecResult, RuleVm, VmError};

/// Fuel profile for a single module execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelProfile {
    /// Module name.
    pub module_name: String,
    /// Fuel budget provided.
    pub fuel_budget: u32,
    /// Fuel consumed (budget - remaining).
    pub fuel_consumed: u32,
    /// Whether execution completed without OutOfFuel.
    pub completed: bool,
}

/// Profile fuel consumption for a single module execution.
pub fn profile_module(module: &BytecodeModule, fuel_budget: u32) -> FuelProfile {
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let result = vm.execute(module, &mut host, fuel_budget);
    let (consumed, completed) = match result {
        Ok(ExecResult::Halted { fuel_remaining }) => {
            (fuel_budget.saturating_sub(fuel_remaining), true)
        }
        Ok(_) => (fuel_budget, true),
        Err(VmError::OutOfFuel) => (fuel_budget, false),
        Err(_) => (fuel_budget, false),
    };
    FuelProfile {
        module_name: module.name.clone(),
        fuel_budget,
        fuel_consumed: consumed,
        completed,
    }
}

/// Render a fuel profile as text.
pub fn render_fuel_profile(profile: &FuelProfile) -> String {
    let status = if profile.completed {
        "completed"
    } else {
        "OUT_OF_FUEL"
    };
    format!(
        "module={:<20} fuel={}/{} ({:.1}%) status={}\n",
        profile.module_name,
        profile.fuel_consumed,
        profile.fuel_budget,
        (profile.fuel_consumed as f64 / profile.fuel_budget.max(1) as f64) * 100.0,
        status
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rule_vm::bytecode::{assemble, Op};

    #[test]
    fn profile_halt_reports_consumed() {
        let code = assemble(&[Op::Nop, Op::Nop, Op::Halt]);
        let module = BytecodeModule {
            name: "test_halt".into(),
            code,
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            logic_hash: [0; 32],
            caps_mask: 0,
        };
        let profile = profile_module(&module, 100);
        assert!(profile.completed);
        // Consumed 3 fuel (NOP + NOP + HALT).
        assert_eq!(profile.fuel_consumed, 3);
    }

    #[test]
    fn profile_infinite_loop_reports_out_of_fuel() {
        let code = assemble(&[Op::Jump(-3)]);
        let module = BytecodeModule {
            name: "test_loop".into(),
            code,
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            logic_hash: [0; 32],
            caps_mask: 0,
        };
        let profile = profile_module(&module, 50);
        assert!(!profile.completed);
        assert_eq!(profile.fuel_consumed, 50);
    }
}
