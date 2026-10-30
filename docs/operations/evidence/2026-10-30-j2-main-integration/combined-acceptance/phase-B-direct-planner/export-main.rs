use serde_json::json;
fn package(source: &str, name: &str) -> String {
    let mut package = content_compiler::compile_source_named(name, source).unwrap();
    let declarations = content_native_loader::bridge_vm_skill_decls(&package).unwrap();
    let generals = [("j2-cross-author",false),("j2-cross-watcher",true)].iter().map(|(name, watcher)| json!({"name":name,"hp":4,"gender":"male","faction":"qun","skills":declarations.decls.iter().filter(|d|d.name.starts_with("j2_watcher")==*watcher).map(|d|json!({"name":d.name})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let characters = json!({"pack":{"name":name,"version":1},"generals":generals});
    let character_bytes = serde_json::to_vec(&characters).unwrap();
    let cards=(name=="danshou-equipment-theft-not-gift").then(||json!({"pack":{"name":name,"version":1,"description":"J2 theft equipment","card_prefix":"j2-theft-"},"cards":[{"name":"gear","kind":"weapon","range":2,"suit":"spade","rank":7,"equip_skills":["j2_theft_gear"]}],"skills":[{"name":"j2_theft_gear","mapping":"host:sha_damage_force_discard"}]}));
    let mut entries = vec![
        json!({"asset_id":"j2:characters","source_hash":blake3::hash(&character_bytes).as_bytes(),"kind":"Data","image_variants":[],"audio_variants":[],"priority":"Lazy","depends_on":[],"platforms":[]}),
    ];
    package
        .asset_paths
        .insert("j2:characters".into(), "assets/character.json".into());
    if let Some(cards) = &cards {
        entries.push(json!({"asset_id":"j2:cards","source_hash":blake3::hash(&serde_json::to_vec(cards).unwrap()).as_bytes(),"kind":"Data","image_variants":[],"audio_variants":[],"priority":"Lazy","depends_on":[],"platforms":[]}));
        package
            .asset_paths
            .insert("j2:cards".into(), "assets/cards.json".into());
    }
    package.asset_manifest=Some(serde_json::from_value(json!({"schema_version":1,"package_id":name,"total_compressed_size":0,"total_decoded_size":0,"entries":entries})).unwrap());
    package.manifest.asset_hash = package.compute_asset_hash();
    package.manifest.logic_hash = package.compute_logic_hash();
    package.validate().unwrap();
    let mut payload = serde_json::to_value(package).unwrap();
    payload["characters"] = characters;
    if let Some(cards) = cards {
        payload["cards"] = cards;
    }
    json!([payload]).to_string()
}


fn main(){let output=std::env::args().nth(1).unwrap();let packs=package("skill nc10_j2_cached(99107899) { active play_phase targets 0 0 { let reds = hand_color_count(2, red); effect resolve_ol_jie_danshou(self); effect deal_damage(self,self,reds+1); } }","nc10-j2-cached");std::fs::write(output,&packs).unwrap();println!("exported real compiled DSL payload, bytes={}",packs.len());}
