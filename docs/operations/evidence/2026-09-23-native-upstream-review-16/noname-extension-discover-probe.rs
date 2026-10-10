use content_runtime::discover_extensions;
use std::path::Path;
fn main() {
    for root in std::env::args().skip(1) {
        let entries = discover_extensions(Path::new(&root)).expect("discovery failed");
        println!("{}: {} discovered", root, entries.len());
        assert!(entries.is_empty());
    }
}
