//! One-time inventory: verify every module in tracked npack JSON files.
use rule_vm::{verify_module, BytecodeModule};

#[test]
fn tracked_package_module_control_flow_inventory() {
    let root = std::env::var("NC04_PACKAGE_ROOT").unwrap();
    let list = std::env::var("NC04_PACKAGE_LIST").unwrap();
    let files = std::fs::read_to_string(list).unwrap();
    let mut packages = 0;
    let mut modules = 0;
    let mut invalid = Vec::new();
    for file in files.lines() {
        packages += 1;
        let bytes = std::fs::read(std::path::Path::new(&root).join(file)).unwrap();
        let package: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for raw in package
            .get("modules")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let module: BytecodeModule = serde_json::from_value(raw.clone()).unwrap();
            modules += 1;
            if let Err(error) = verify_module(&module) {
                invalid.push(format!("{file} [{}]: {error}", module.name));
            }
        }
    }
    println!(
        "tracked npack files={packages} modules={modules} invalid={}",
        invalid.len()
    );
    for item in &invalid {
        println!("{item}");
    }
    assert!(
        invalid.is_empty(),
        "tracked modules failed control-flow verification"
    );
}
