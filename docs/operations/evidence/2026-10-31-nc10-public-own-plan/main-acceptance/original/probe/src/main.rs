use engine_native::{Engine,Phase,GameAction,GameCommand,CommandOutcome};
use engine_native_wasm::{NativeSession,hex_encode};
use std::{fs,path::PathBuf};
fn main(){
 let args:Vec<_>=std::env::args().collect();let out=PathBuf::from(&args[1]);
 if args.get(2).map(String::as_str)==Some("compare") {
  let mut count=0;
  for name in ["park-0","park-1","own-cancel","shan-auto","shan-direct","shan-cancel","shan-pass"] {
   let a=engine_native::archive::decode_archive(&fs::read(out.join(format!("wasm-{name}.ngsa"))).unwrap()).unwrap();
   let b=engine_native::archive::decode_archive(&fs::read(out.join(format!("native-{name}.ngsa"))).unwrap()).unwrap();
   assert_eq!(a,b,"entire snapshot incl RNG/resolution/events: {name}");
   fs::write(out.join(format!("snapshot-{name}.txt")),format!("{a:#?}")).unwrap();count+=1;
  }
  println!("full canonical snapshots equal={count}; no ignored fields");return;
 }
 let mut e=Engine::new(2303,3);e.debug_set_turn(0,Phase::Play,1);for seat in 0..3{e.debug_clear_hand(seat);}
 e.debug_place(1,"shan");let sha=e.debug_place(0,"sha");
 assert!(matches!(e.submit(GameCommand{actor:0,expected_revision:e.revision(),dedup_id:1,action:GameAction::PlayCard{card:sha,targets:vec![1]}}),CommandOutcome::Pending{..}));
 fs::write(out.join("shan.ngsa"),engine_native::archive::encode_archive(&e.save())).unwrap();
 let ch=e.open_choice().unwrap();let card_index=ch.spec.options.iter().position(|x|matches!(x,engine_native::ChoiceOption::Card(_))).unwrap() as u32;
 fs::write(out.join("shan-choice.txt"),format!("{ch:#?}")).unwrap();let hex=hex_encode(&engine_native::archive::encode_archive(&e.save()));let mut replies=serde_json::Map::new();
 for mode in ["auto","direct","cancel","pass"] {
  let mut session=NativeSession::new(3,0).unwrap();session.restore_hex_checked(&hex).unwrap();let rev=session.revision();let restored=session.save_hex();let restored_hash=session.state_hash_hex();let restored_log=session.log_len();
  let reply=if mode=="auto"{session.auto_step().unwrap()}else{session.answer(1,ch.id,if mode=="direct"{vec![card_index]}else if mode=="pass"{vec![ch.spec.options.iter().position(|x|matches!(x,engine_native::ChoiceOption::Pass)).unwrap() as u32]}else{vec![]}).unwrap()};fs::write(out.join(format!("shan-{mode}-reply.txt")),&reply).unwrap();if mode=="cancel" {assert!(!ch.spec.allow_cancel);assert!(!ch.spec.accepts(&[]));assert_eq!(session.revision(),rev);assert_eq!(session.save_hex(),restored);assert_eq!(session.state_hash_hex(),restored_hash);assert_eq!(session.log_len(),restored_log);}else{assert_eq!(session.revision(),rev+1);}
  let parsed:serde_json::Value=serde_json::from_str(&reply).unwrap();let summary:serde_json::Value=if mode=="auto"{serde_json::from_str(parsed["outcome"].as_str().unwrap()).unwrap()}else{parsed};if mode=="cancel"{assert_eq!(summary["outcome"],"rejected");assert_eq!(summary["reason"],"InvalidChoice");}else{assert!(matches!(summary["outcome"].as_str(),Some("accepted"|"pending")));}
  fs::write(out.join(format!("native-shan-{mode}.ngsa")),decode(&session.save_hex())).unwrap();replies.insert(mode.into(),serde_json::json!({"reply":serde_json::from_str::<serde_json::Value>(&reply).unwrap(),"revision":session.revision(),"hash":session.state_hash_hex(),"codes":serde_json::from_str::<serde_json::Value>(&session.public_event_codes_json(0)).unwrap()}));
 }
 assert_eq!(fs::read(out.join("native-shan-auto.ngsa")).unwrap(),fs::read(out.join("native-shan-direct.ngsa")).unwrap());
 for side in 0..2 {
  let archive=fs::read(out.join(format!("main-own-cases/park-{side}.ngsa"))).unwrap();fs::write(out.join(format!("park-{side}.ngsa")),&archive).unwrap();
  let mut session=NativeSession::new(3,0).unwrap();session.restore_hex_checked(&hex_encode(&archive)).unwrap();let mut baseline=engine_native::archive::decode_archive(&archive).unwrap();baseline.auth_generation+=1;assert_eq!(engine_native::archive::decode_archive(&decode(&session.save_hex())).unwrap(),baseline,"restore only rotates auth generation");let reply=session.auto_step().unwrap();assert_eq!(serde_json::from_str::<serde_json::Value>(&serde_json::from_str::<serde_json::Value>(&reply).unwrap()["outcome"].as_str().unwrap()).unwrap()["outcome"],"pending");
  let after=decode(&session.save_hex());fs::write(out.join(format!("native-park-{side}.ngsa")),after).unwrap();
  replies.insert(format!("park-{side}"),serde_json::json!({"reply":serde_json::from_str::<serde_json::Value>(&reply).unwrap(),"revision":session.revision(),"hash":session.state_hash_hex(),"codes":serde_json::from_str::<serde_json::Value>(&session.public_event_codes_json(0)).unwrap()}));
 }
 let archive=fs::read(out.join("park-0.ngsa")).unwrap();let mut session=NativeSession::new(3,0).unwrap();session.restore_hex_checked(&hex_encode(&archive)).unwrap();let own:serde_json::Value=serde_json::from_str(&session.open_choice_json(1).unwrap()).unwrap();assert_eq!(own["allowCancel"],true);let rev=session.revision();let reply=session.answer(1,own["id"].as_u64().unwrap(),vec![]).unwrap();let result:serde_json::Value=serde_json::from_str(&reply).unwrap();assert_eq!(result["outcome"],"accepted");assert_eq!(session.revision(),rev+1);assert_eq!(session.open_choice_json(1).unwrap(),"null");fs::write(out.join("native-own-cancel.ngsa"),decode(&session.save_hex())).unwrap();replies.insert("own-cancel".into(),serde_json::json!({"reply":result,"revision":session.revision(),"hash":session.state_hash_hex(),"codes":serde_json::from_str::<serde_json::Value>(&session.public_event_codes_json(0)).unwrap()}));
 fs::write(out.join("native-probe-results.json"),serde_json::to_string_pretty(&replies).unwrap()).unwrap();println!("native Shan card-first/direct/cancel and both park formal Pending generated");
}
fn decode(hex:&str)->Vec<u8>{hex.as_bytes().chunks_exact(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect()}
