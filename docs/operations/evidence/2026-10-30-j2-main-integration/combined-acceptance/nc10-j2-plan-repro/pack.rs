use engine_native::content::extension::{ExtensionDef, ExtensionPack};
pub fn pack(source: &str) -> ExtensionPack {
    let p = content_compiler::compile_source_named("j2-review", source).unwrap();
    let decls = content_native_loader::bridge_vm_skill_decls(&p)
        .unwrap()
        .decls;
    let known = decls.iter().map(|d| d.name.clone()).collect();
    let roles = [("author", false), ("watcher", true)];
    let generals=roles.iter().map(|(name,watcher)|serde_json::json!({"name":name,"hp":4,"faction":"qun","gender":"male","skills":decls.iter().filter(|d|d.name.starts_with("j2_watcher")==*watcher).map(|d|serde_json::json!({"name":d.name})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let generals = content_native_loader::bridge_generals_with_known(
        &serde_json::json!({"pack":{"name":"j2-review","version":1},"generals":generals})
            .to_string(),
        &known,
    )
    .unwrap()
    .generals;
    ExtensionPack {
        def: ExtensionDef {
            name: "j2-review",
            version: 1,
            description: "review",
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}
