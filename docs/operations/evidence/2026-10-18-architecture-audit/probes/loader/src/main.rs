#![allow(dead_code)]
#[path = "/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/game-server/src/native.rs"]
mod native;

use native::NativeContent;

const DSL: &str = "skill audit_loader_probe(990001) {\n  trigger phase_end owner_is_target {\n    if hp(self) < max_hp(self) {\n      effect recover_hp(self, 1);\n    } else {\n      effect draw_cards(self, 1);\n    }\n  }\n}\n";

fn install_vm_package(case: &str) -> (tempfile::TempDir, std::path::PathBuf, NativeContent) {
    let dir = tempfile::Builder::new().prefix(case).tempdir_in("/tmp").unwrap();
    let source = dir.path().join("source");
    let dropin = source.join("audit-vm-pack");
    std::fs::create_dir_all(&dropin).unwrap();
    let package = content_compiler::compile_source_named("audit-disk-vm", DSL).expect("compile fixture");
    std::fs::write(dropin.join("package.npack.json"), serde_json::to_vec(&package).unwrap()).unwrap();
    let content = NativeContent::open(source, dir.path().join("store")).expect("open store");
    let installed = content.import().expect("import VM package");
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].package_id, "audit-disk-vm");
    assert_eq!(content.load("audit-disk-vm").unwrap().pack.vm_skills.len(), 1);
    (dir, dropin, content)
}

fn main() {
    let (dir, dropin, content) = install_vm_package("loader-probe-delete-");
    std::fs::remove_dir_all(&dropin).unwrap();
    let installed = content.installed().expect("installed metadata must survive source removal");
    assert_eq!(installed.len(), 1);
    assert!(dir.path().join("store").is_dir());
    match content.load("audit-disk-vm") {
        Err(error) => println!("REPRO 1 CONFIRMED: installed_count={} store_exists=true load_failed_after_source_removal={error}", installed.len()),
        Ok(_) => panic!("REPRO 1 NOT CONFIRMED: load succeeded after source removal"),
    }

    let (_dir, dropin, content) = install_vm_package("loader-probe-cards-");
    let before = content.load("audit-disk-vm").unwrap().pack.vm_skills.len();
    std::fs::create_dir_all(dropin.join("assets")).unwrap();
    std::fs::copy(
        "/Users/ruyimg/Documents/Claude/sanguosha/noname-next/packages/extensions/huanle-upstream-noname/assets/cards.json",
        dropin.join("assets/cards.json"),
    ).unwrap();
    let after = content.load("audit-disk-vm").expect("load with cards JSON");
    assert_eq!(before, 1);
    assert_eq!(after.pack.vm_skills.len(), 0);
    assert!(after.pack.card_count() > 0);
    println!("REPRO 2 CONFIRMED: imported_vm_skills={before} after_adding_cards_json_vm_skills={} cards={} package_id={} bridged_pack_name={}", after.pack.vm_skills.len(), after.pack.card_count(), after.package_id, after.pack.def.name);
    println!("Both counterexamples reproduced against repository native.rs without modifying project source.");
}
