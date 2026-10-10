use engine_native::{Engine, GameCommand, GameAction, CommandOutcome, ChoiceOption, Phase, EventView, LogEvent};
use engine_native::archive::{encode_archive,decode_archive};
#[path="../../../../crates/engine-native/examples/host_assembler.rs"]
mod assembler;
fn submit(e:&mut Engine,actor:u8,action:GameAction)->CommandOutcome {let s=e.save(); e.submit(GameCommand{actor,action,expected_revision:e.revision(),dedup_id:s.last_dedup+1})}
fn answer(e:&mut Engine,selection:Vec<ChoiceOption>)->CommandOutcome {let c=e.open_choice().unwrap().clone();submit(e,c.observer,GameAction::Answer{choice_id:c.id,selection})}
fn main(){
 let mut a=Engine::new(443,4);let token=a.issue_token(0);let mut b=Engine::new(443,4);let token_b=b.issue_token(0);
 assert_eq!(token,token_b);let cmd=GameCommand{actor:0,expected_revision:b.revision(),dedup_id:1,action:GameAction::NextPhase};
 let out=b.submit_authed(token,cmd);assert!(!matches!(out,CommandOutcome::Rejected(_)));println!("AUTH_CROSS_ENGINE: same seed and issuance order generate equal tokens; A token accepted by B: {out:?}");
 let snap=a.save();let mut restored=Engine::restore(&snap).unwrap();let reissued=restored.issue_token(0);assert_eq!(token,reissued);
 let cmd=GameCommand{actor:0,expected_revision:restored.revision(),dedup_id:1,action:GameAction::NextPhase};let out=restored.submit_authed(token,cmd);assert!(!matches!(out,CommandOutcome::Rejected(_)));println!("AUTH_RESTART: old token becomes valid after reissue at restored state: {out:?}");
 let orig=encode_archive(&snap);let mut bad=orig.clone();bad[6..14].copy_from_slice(&999u64.to_le_bytes());let back=decode_archive(&bad).unwrap();assert_eq!(back,snap);assert!(Engine::restore(&back).is_ok());println!("ARCHIVE_HEADER: ruleset header 24->999 accepted, decoded state still identical");
 let mut bad=orig.clone();bad[14..22].copy_from_slice(&u64::MAX.to_le_bytes());let panic=std::panic::catch_unwind(||decode_archive(&bad)).is_err();assert!(panic);println!("ARCHIVE_LENGTH: u64::MAX payload length panics in debug decoder instead of Err");
 let mut bad=snap.clone();bad.core.players[0].general_slots=[Some(u16::MAX),None];bad.core.players[0].revealed_generals=[true,false];assert!(Engine::restore(&bad).is_ok());println!("SNAPSHOT_GENERAL: invalid general index 65535 with revealed=true accepted");
 let mut bad=snap.clone();let duplicate=bad.core.players[0].hand[0];bad.core.discard.push(duplicate);assert!(Engine::restore(&bad).is_ok());println!("SNAPSHOT_OWNERSHIP: same physical card in hand and discard accepted");
 let mut q=Engine::new(443,4);q.set_seat_quota(Some(1));let first=submit(&mut q,0,GameAction::NextPhase);let phase=q.turn().phase;let second=submit(&mut q,0,GameAction::NextPhase);assert!(matches!(second,CommandOutcome::Rejected(engine_native::RejectReason::QuotaExhausted)));assert_eq!(q.turn().phase,phase);assert_eq!(q.turn().round,1);println!("QUOTA_PROGRESS: first={first:?}, next={second:?}; round=1 current seat=0 phase={phase:?}, automatic round reset cannot advance without host intervention");
 let (assembled,dealt)=assembler::assemble_kingdoms(4,42,7);assert_eq!(assembled.state_core().skills.len(),0);println!("HOST_ASSEMBLER: {} general slots filled but active skill instances={} (metadata only)",dealt.len(),assembled.state_core().skills.len());
 let mut e=Engine::new(901,3);e.debug_grant(0,"观星");for p in 0..3{e.debug_clear_hand(p)}let mut found=false;
 for _ in 0..128 {
   let actor=e.turn().current;
   match submit(&mut e,actor,GameAction::NextPhase) {
     CommandOutcome::Accepted{..}=>{},
     CommandOutcome::Pending{choice}=>{
       assert_eq!(choice.spec.prompt,"trigger_confirm");answer(&mut e,vec![ChoiceOption::Confirm]);
       let c=e.open_choice().unwrap().clone();assert_eq!(c.spec.prompt,"guanxing_arrange");
       let card=c.spec.options.iter().find(|x|matches!(x,ChoiceOption::Card(_))).copied().unwrap();
       let out=answer(&mut e,vec![card]);assert!(!matches!(out,CommandOutcome::Rejected(_)));
       let leaked=e.ai_session(1).events().into_iter().find(|x|matches!(x,EventView::Public(LogEvent::ChoiceAnswered{id,..}) if *id==c.id));
       assert!(leaked.is_some());println!("AI_HIDDEN_CHOICE: opponent AI receives private guanxing answer {leaked:?}; selected={card:?}");found=true;break;
     },other=>panic!("guanxing setup: {other:?}")
   }
 }
 assert!(found);let _=Phase::Judge;
 println!("REVIEW12_PROBES_COMPLETE");
}
