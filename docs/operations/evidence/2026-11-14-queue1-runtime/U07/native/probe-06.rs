//! External runtime fixture probe. No product code or equivalence signoff.
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, CardKind};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
static SEQ: AtomicU64=AtomicU64::new(1);
fn act(e:&mut Engine,actor:u8,action:GameAction)->CommandOutcome {
 e.submit(GameCommand{actor,expected_revision:e.revision(),dedup_id:SEQ.fetch_add(1,Ordering::Relaxed),action})
}
fn face(e:&Engine,c:u32)->Value {let d=&e.catalog().cards[c as usize]; json!({"id":c,"name":d.name,"kind":format!("{:?}",d.kind),"suit":format!("{:?}",d.suit).to_lowercase(),"rank":d.rank})}
fn players(e:&Engine)->Value {json!((0..8).map(|s|{let p=e.state_core().player(s);json!({"seat":s,"hp":p.hp,"max_hp":p.max_hp,"alive":p.alive,"hand":p.hand.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),"equips":p.equips.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),"judge":p.judge.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),"fuchou_marked":p.fuchou_marked,"skills":e.state_core().skills.range((s,0)..(s,u16::MAX)).map(|((_,d),_)|e.catalog().skills[*d as usize].name).collect::<Vec<_>>()})}).collect::<Vec<_>>())}
fn snapshot(e:&Engine)->Value {json!({"players":players(e),"deck_draw_order":e.state_core().deck.iter().rev().map(|&c|face(e,c)).collect::<Vec<_>>(),"discard":e.state_core().discard.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),"table":e.state_core().table.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),"turn":format!("{:?}",e.turn()),"state_hash":format!("{:?}",e.state_hash())})}
fn take(e:&mut Engine,c:u32,seat:u8,equip:bool){e.state_core_mut().deck.retain(|&x|x!=c);if equip{e.state_core_mut().player_mut(seat).equips.push(c);}else{e.state_core_mut().player_mut(seat).hand.push(c);}}
fn by(e:&Engine,name:&str,n:usize)->u32{e.catalog().cards.iter().enumerate().filter(|(_,d)|d.name==name || (name=="sanjian" && d.name.contains("sanjian"))).nth(n).map(|(i,_)|i as u32).unwrap_or_else(||panic!("missing {name} {n}"))}
fn answer(e:&mut Engine,pack:&ExtensionPack,sel:Vec<ChoiceOption>,trace:&mut Vec<Value>){
 let c=e.open_choice().unwrap().clone();let bytes=encode_archive(&e.save());let archive=std::env::var("U07_ARCHIVE_DIR").ok().map(|dir|{let p=format!("{dir}/pending-{:03}-{}.ngsa",SEQ.load(Ordering::Relaxed),c.spec.prompt);std::fs::write(&p,&bytes).unwrap();p});let mut r=Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(),&[pack]).unwrap();
 assert_eq!(e.state_hash(),r.state_hash());let old=e.log().len();
 let out=act(e,c.observer,GameAction::Answer{choice_id:c.id,selection:sel.clone()});
 let resumed=act(&mut r,c.observer,GameAction::Answer{choice_id:c.id,selection:sel.clone()});
 assert!(!matches!(out,CommandOutcome::Rejected(_)),"probe answer rejected: {} {sel:?} {out:?}",c.spec.prompt);assert_eq!(format!("{out:?}"),format!("{resumed:?}"));assert_eq!(e.state_hash(),r.state_hash());assert_eq!(&e.log()[old..],r.log());
 trace.push(json!({"prompt":c.spec.prompt,"observer":c.observer,"min":c.spec.min,"max":c.spec.max,"allow_cancel":c.spec.allow_cancel,"options":format!("{:?}",c.spec.options),"selection":format!("{sel:?}"),"archive_bytes":bytes.len(),"archive_file":archive,"outcome":format!("{out:?}"),"resume_state_hash_log_equal":true,"after":snapshot(e)}));
}
fn run(case:&str,pack:&ExtensionPack){
 let mut e=Engine::new_with_extensions(2026111407,8,&[pack]).unwrap();e.state_core_mut().skills.clear();
 for s in 0..8{let p=e.state_core_mut().player_mut(s);p.hand.clear();p.equips.clear();p.judge.clear();p.hp=8;p.max_hp=8;}
 let cost=by(&e,"sha",0);let attack=by(&e,"sha",1);let spare=by(&e,"sha",2);let hand=by(&e,"shan",0);let other=by(&e,"tao",1);let weapon=by(&e,"sanjian",0);let armor=by(&e,"armor",0);
 let reserved=[cost,attack,spare,hand,other,weapon,armor];
 let mut deck:Vec<_>=e.catalog().cards.iter().enumerate().filter(|(i,d)|!reserved.contains(&(*i as u32)) && matches!(d.kind,CardKind::Sha|CardKind::Shan|CardKind::Tao)).take(16).map(|(i,_)|i as u32).collect();
 let drawkind=if case.contains("draw-sha"){CardKind::Sha}else{CardKind::Tao};
 let draw=e.catalog().cards.iter().enumerate().find(|(i,d)|!reserved.contains(&(*i as u32)) && d.kind==drawkind).map(|(i,_)|i as u32).unwrap();
 deck.retain(|&c|c!=draw);deck.insert(0,draw);let rest:Vec<_>=(0..e.catalog().cards.len() as u32).filter(|c|!deck.contains(c)).collect();deck.extend(rest);e.state_core_mut().deck=deck.into_iter().rev().collect();e.state_core_mut().discard.clear();e.state_core_mut().table.clear();
 let (actor,target)=if case.starts_with("youzhan"){(1,if case.ends_with("far"){2}else{0})}else{(0,1)};
 e.debug_set_turn(actor,Phase::Play,1);
 if case.starts_with("choudu"){
  e.debug_grant(0,"choudu");
  if !case.ends_with("empty"){take(&mut e,cost,0,false);take(&mut e,weapon,0,true);e.debug_grant(0,"wd_sanjian");}
  take(&mut e,hand,1,false);take(&mut e,other,1,false);take(&mut e,spare,2,false);
 }else if case.starts_with("fuchou"){
  e.debug_grant(1,"fuchou");e.debug_grant(1,"fuchou2");take(&mut e,attack,0,false);
  if !case.ends_with("empty"){take(&mut e,hand,1,false);take(&mut e,weapon,1,true);e.debug_grant(1,"wd_sanjian");}
 }else{
  e.debug_grant(0,"youzhan");take(&mut e,attack,1,false);
  if !case.ends_with("no-equip"){take(&mut e,armor,0,false);take(&mut e,weapon,0,true);e.debug_grant(0,"wd_sanjian");}
  if case.contains("counter"){take(&mut e,spare,1,false);}if case.contains("dodge"){take(&mut e,hand,0,false);}
 }
 let mut trace=vec![];let initial=snapshot(&e);let initial_log=e.log().len();
 let out=if case.starts_with("choudu"){let skill=e.catalog().skill_by_name("choudu").unwrap();act(&mut e,0,GameAction::ActivateSkill{skill})}else{act(&mut e,actor,GameAction::PlayCard{card:attack,targets:vec![target]})};
 let mut repetitions=0;
 while let Some(c)=e.open_choice().cloned(){
  repetitions+=1;assert!(repetitions<40,"loop {case}");
  let sel=match c.spec.prompt {
   "wudian_choudu_card"=>if case.ends_with("cancel-card"){vec![]}else{vec![ChoiceOption::Card(if case.contains("equip"){weapon}else{cost})]},
   "wudian_choudu_target"=>if case.ends_with("cancel-target"){vec![]}else{vec![ChoiceOption::Player(if case.contains("target2"){2}else{1})]},
   "wudian_diaobing_swap"=>if case.contains("swap-a"){vec![ChoiceOption::Card(hand)]}else if case.contains("swap-b"){vec![ChoiceOption::Card(other)]}else{vec![]},
   "wudian_fuchou_cancel"=>if case.ends_with("cancel"){vec![]}else{vec![ChoiceOption::Card(if case.contains("equip"){weapon}else{hand})]},
   "wudian_youzhan_discard"=>if case.ends_with("cancel") || repetitions>1{vec![]}else{vec![ChoiceOption::Card(if case.contains("equip-"){weapon}else{armor})]},
   "wudian_youdishenru_sha"=>if case.contains("counter-use") || (case.contains("dodge") && repetitions==2){vec![ChoiceOption::Card(spare)]}else{vec![]},
   "respond_shan"=>if case.contains("dodge") && e.state_core().player(c.observer).hand.contains(&hand){vec![ChoiceOption::Card(hand)]}else{vec![ChoiceOption::Pass]},
   p=>panic!("unexpected prompt {p} in {case}")};
  answer(&mut e,pack,sel,&mut trace);
 }
 let after_attack=snapshot(&e);
 let mut phase_outcomes=vec![];
 if case.contains("settle"){
  for _ in 0..3{phase_outcomes.push(format!("{:?}",act(&mut e,0,GameAction::NextPhase)));
   while let Some(c)=e.open_choice().cloned(){answer(&mut e,pack,if c.spec.min>0 && !c.spec.allow_cancel{c.spec.options.iter().take(c.spec.min).copied().collect()}else{vec![]},&mut trace);}
  }
 }
 println!("{}",json!({"case":case,"initial":initial,"initial_outcome":format!("{out:?}"),"actor":actor,"target":target,"cards":{"cost":cost,"attack":attack,"spare":spare,"hand":hand,"other":other,"weapon":weapon,"armor":armor},"choices":trace,"after_attack":after_attack,"final":snapshot(&e),"phase_outcomes":phase_outcomes,"settled":e.open_choice().is_none(),"log":e.log()[initial_log..].iter().map(|x|format!("{x:?}")).collect::<Vec<_>>()}));
}
fn main(){let root=std::env::var("NONAME_REPO_ROOT").unwrap();let s=std::fs::read_to_string(format!("{root}/packages/extensions/wudian-upstream-noname/assets/cards.json")).unwrap();let mut p=content_native_loader::bridge_pack(&s.replace("\r\n","\n")).unwrap().pack;content_native_loader::attach_formal_package_upstream_rules("upstream-noname","wudian",&mut p,&[]);if !p.cards.iter().any(|c|c.name=="diaobingqianjiang"){p.cards.push(CardDef::basic("diaobingqianjiang",CardKind::Wuzhong));}
 for c in ["choudu-top-return","choudu-target1","choudu-target2","choudu-swap-a","choudu-swap-b","choudu-equip-target1","choudu-cancel-card","choudu-cancel-target","choudu-empty","fuchou-hand","fuchou-equip","fuchou-cancel","fuchou-empty","fuchou-settle-no-sha","fuchou-settle-draw-sha","fuchou-settle-draw-sha-use","youzhan-hand-stop","youzhan-equip-stop","youzhan-cancel","youzhan-far","youzhan-no-equip","youzhan-counter-use","youzhan-counter-dodge-stop"]{run(c,&p);}}
