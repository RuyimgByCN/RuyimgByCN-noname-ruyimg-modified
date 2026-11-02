//! Approved count widths through verified packages and restored final body asks.
#[path = "../src/native.rs"]
mod native;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use serde_json::json;
fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c=e.open_choice().unwrap();
    e.submit(GameCommand {actor:c.observer,expected_revision:e.revision(),dedup_id:e.save().last_dedup+1,action:GameAction::Answer {choice_id:c.id,selection}})
}
fn finish(e: &mut Engine) {
    for _ in 0..500 {
        if e.open_choice().is_some() {assert!(!matches!(answer(e,vec![]),CommandOutcome::Rejected(_)));}
        else if e.save().segment_open {e.resume();}
        else {return;}
    }
    panic!("wide continuation did not finish");
}
#[test]
fn c9j_verified_255_256_257_final_asks_restore_then_select_or_cancel() {
    for count in [255usize,256,257] {
        let temp=tempfile::tempdir().unwrap();
        let root=temp.path().join("packages");
        let dir=root.join("c9j-wide");std::fs::create_dir_all(dir.join("assets")).unwrap();
        let cards=serde_json::to_vec(&json!({"pack":{"name":"c9j-wide","version":1,"description":"supported physical cards","card_prefix":"c9j_"},"cards":(0..count).map(|i|json!({"name":format!("c9j_sha_{i}"),"kind":"sha","suit":"spade","rank":1})).collect::<Vec<_>>()})).unwrap();
        let mut package=content_compiler::compile_source_named("c9j-wide","skill c9j_anchor(99109701) locked { trigger phase_play_end owner_is_target { effect draw_cards(self,1); } }").unwrap();
        package.manifest.content_schema_version=package.required_content_schema();
        let mut value=serde_json::to_value(package).unwrap();
        value["asset_manifest"]=json!({"schema_version":1,"package_id":"c9j-wide","total_compressed_size":0,"total_decoded_size":0,"entries":[{"asset_id":"c9j-wide:cards","source_hash":blake3::hash(&cards).as_bytes(),"kind":"Data","image_variants":[],"audio_variants":[],"priority":"Lazy","depends_on":[],"platforms":[]}]});
        value["asset_paths"]=json!({"c9j-wide:cards":"assets/cards.json"});
        let mut package:content_schema::ContentPackage=serde_json::from_value(value).unwrap();
        package.manifest.logic_hash=package.compute_logic_hash();package.manifest.asset_hash=package.compute_asset_hash();package.validate().unwrap();
        std::fs::write(dir.join("package.npack.json"),package.to_json_bytes().unwrap()).unwrap();
        std::fs::write(dir.join("assets/cards.json"),cards).unwrap();
        let content=native::NativeContent::open(root,temp.path().join("store")).unwrap();content.import().unwrap();
        let loaded=content.load("c9j-wide").unwrap();assert_eq!(loaded.pack.cards.len(),count);
        let pack=&loaded.pack;
        let mut e=Engine::new_with_extensions(19001,3,&[pack]).unwrap();
        for seat in 0..3 {e.debug_clear_hand(seat);}
        e.debug_set_max_hp(0,1000);e.debug_set_hp(0,1000);e.debug_grant(1,"章武");
        for card in &pack.cards {e.debug_move_named_card_to_hand(1,card.name);}
        e.debug_deal_damage(Some(0),1,1,engine_native::event::DamageNature::Normal);
        let choice=e.open_choice().expect("actual final Zhangwu body ask").clone();
        assert_eq!((choice.spec.prompt,choice.spec.max),("c331_zhangwu_holder",count));
        let plan=e.plan_action(1,&engine_native::ai::BoundedPlanner::new(4));
        assert_eq!(plan.sim_faults,0);
        let mut ai=Engine::restore_with_extensions(&e.save(),&[pack]).unwrap();
        let out=ai.submit(GameCommand{actor:1,expected_revision:ai.revision(),dedup_id:ai.save().last_dedup+1,action:plan.action});
        assert!(!matches!(out,CommandOutcome::Rejected(_)));finish(&mut ai);
        for cancel in [false,true] {
            let snapshot=e.save();
            let mut original=e.clone();
            let mut typed=Engine::restore_with_extensions(&snapshot,&[pack]).unwrap();
            let decoded=engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snapshot)).unwrap();
            let mut bytes=Engine::restore_with_extensions(&decoded,&[pack]).unwrap();
            for branch in [&mut original,&mut typed,&mut bytes] {
                let before=branch.save();
                assert!(matches!(answer(branch,vec![choice.spec.options[0],choice.spec.options[0]]),CommandOutcome::Rejected(_)));
                assert_eq!(branch.save(),before,"bad input is atomic from restored baseline");
                let selection=if cancel {vec![]} else {choice.spec.options.clone()};
                assert!(!matches!(answer(branch,selection),CommandOutcome::Rejected(_)));
                finish(branch);
                assert_eq!(branch.debug_hand(1).len(),if cancel {count} else {0});
                assert_eq!(branch.debug_hp(0),if cancel {1000} else {1000-count as i32});
            }
            let normalize=|e:&Engine| {let mut s=e.save();s.auth_generation=0;s};
            assert_eq!(normalize(&original),normalize(&typed));assert_eq!(normalize(&original),normalize(&bytes));
            assert_eq!(original.state_hash(),typed.state_hash());assert_eq!(original.state_hash(),bytes.state_hash());
            assert!(original.log().ends_with(typed.log()));assert!(original.log().ends_with(bytes.log()));
        }
    }
}
