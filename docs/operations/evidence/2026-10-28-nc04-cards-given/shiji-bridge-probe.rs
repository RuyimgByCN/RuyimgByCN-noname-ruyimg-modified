// 探针（不入产品）：真实 shiji npack 的事件/滤镜映射面。产品装载顺序中
// ABI 窗口（host=753 vs 包 756）先于事件映射拒绝——本探针临时把包 ABI 视作
// 753，仅用于回答「事件/滤镜层还剩什么未映射」，不改变包内容锁与产品行为。
#[test]
fn real_shiji_npack_event_filter_surface() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/shiji/shiji.npack.json");
    let bytes = std::fs::read(path).unwrap();
    let mut pkg = content_schema::ContentPackage::from_json_bytes(&bytes).unwrap();
    let mut events = std::collections::BTreeMap::new();
    let mut filters = std::collections::BTreeMap::new();
    for t in &pkg.trigger_index {
        *events.entry(t.event.clone()).or_insert(0usize) += 1;
        *filters.entry(t.filter.clone()).or_insert(0usize) += 1;
    }
    println!("SHIJI_EVENTS {}", serde_json::to_string(&events).unwrap());
    println!("SHIJI_FILTERS {}", serde_json::to_string(&filters).unwrap());
    pkg.manifest.rule_vm_abi_version = 753;
    match content_native_loader::bridge_vm_skill_decls(&pkg) {
        Ok(out) => println!("SHIJI_BRIDGE_OK decls={}", out.decls.len()),
        Err(e) => println!("SHIJI_BRIDGE_ERR {e:?}"),
    }
}
