use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine_with_original_presentation,
    start_locked_engine_with_original_presentation, validate_and_freeze_raw_package,
    LockedStartMode, VerifiedLockedContent,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction as A, GameCommand, Phase};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn face(e: &Engine, id: u32) -> Value {
    let c = &e.catalog().cards[id as usize];
    let up = match c.kind {
        CardKind::FireSha | CardKind::LeiSha => "sha",
        CardKind::HorsePlus => "jueying", CardKind::HorseMinus => "chitu",
        CardKind::Weapon => match c.name { "weapon2" => "qinggang", "weapon2b" => "cixiong", "weapon3" => "qinglong", _ => "zhuge" },
        CardKind::Armor => if c.name == "th_taipingyaoshu" { "th_taipingyaoshu" } else { "bagua" },
        _ => c.kind.key(),
    };
    json!({"id":id,"name":c.name,"kind":c.kind.key(),"upstream_name":up,
        "suit":format!("{:?}",c.suit).to_lowercase(),"rank":c.rank,
        "nature":match c.kind { CardKind::FireSha=>Some("fire"),CardKind::LeiSha=>Some("thunder"),_=>None }})
}
fn state(e: &Engine) -> Value {
    let c = e.state_core();
    let players: Vec<_> = c.players.iter().enumerate().map(|(s,p)| {
        let skills:Vec<_>=c.skills.iter().filter(|((o,_),_)|usize::from(*o)==s).map(|((_,d),i)|json!({"name":e.catalog().skill_name(*d),"enabled":i.enabled,"once_used":i.once_used})).collect();
        let flags:Vec<_>=c.vm_aux.flags.iter().filter(|((o,k),_)|usize::from(*o)==s&&k.starts_with("th_")).map(|((_,k),v)|json!({"key":k,"value":v})).collect();
        let piles:Vec<_>=c.vm_aux.piles.iter().filter(|((o,_),_)|usize::from(*o)==s).map(|((_,k),v)|json!({"key":k,"cards":v.iter().map(|x|face(e,*x)).collect::<Vec<_>>()})).collect();
        json!({"seat":s,"hp":p.hp,"max_hp":p.max_hp,"alive":p.alive,"linked":p.linked,"face_down":p.face_down,"hujia":p.hujia,
          "hand":p.hand.iter().map(|x|face(e,*x)).collect::<Vec<_>>(),"equips":p.equips.iter().map(|x|face(e,*x)).collect::<Vec<_>>(),"judge":p.judge.iter().map(|x|face(e,*x)).collect::<Vec<_>>(),"skills":skills,"flags":flags,"piles":piles})
    }).collect();
    json!({"players":players,"deck_draw_order":c.deck.iter().rev().map(|x|face(e,*x)).collect::<Vec<_>>(),
       "discard":c.discard.iter().map(|x|face(e,*x)).collect::<Vec<_>>(),"table":c.table.iter().map(|x|face(e,*x)).collect::<Vec<_>>(),
       "turn":format!("{:?}",e.turn()),"state_hash":format!("{:?}",e.state_hash())})
}
fn flag(e:&mut Engine,key:&str,v:u32){ e.state_core_mut().vm_aux.flags.insert((0,key.to_owned()),v); }
fn take(e:&mut Engine,id:u32){e.state_core_mut().deck.retain(|c|*c!=id);e.state_core_mut().player_mut(0).hand.push(id);}
fn card(e:&Engine,k:CardKind)->u32{e.catalog().cards.iter().position(|c|c.kind==k).unwrap()as u32}
fn grant(e:&mut Engine,name:&str){assert!(e.catalog().skill_by_name(name).is_some(),"missing formal skill {name}");e.debug_grant(0,name);}
fn submit(e:&mut Engine,action:A,dedup:u64)->CommandOutcome{e.submit(GameCommand{actor:0,expected_revision:e.revision(),dedup_id:dedup,action})}

fn answer(e:&mut Engine,v:&VerifiedLockedContent,base:&Path,case:&str,selection:Vec<O>,seq:&mut u64,trace:&mut Vec<Value>){
    let choice=e.open_choice().unwrap().clone();let before=e.save();let bytes=encode_archive(&before);
    let file=base.join(format!("{case}-pending-{:02}-{}.ngsa",trace.len(),choice.spec.prompt.replace(':',"-")));fs::write(&file,&bytes).unwrap();
    let decoded=decode_archive(&bytes).unwrap();let restored=restore_locked_engine_with_original_presentation(&decoded,v,None);
    let mut r=restored.unwrap_or_else(|x|panic!("restore {case} {}: {x:?}",choice.spec.prompt));
    let mut normalized=r.save();normalized.auth_generation=before.auth_generation;assert_eq!(normalized,before,"full snapshot before answer");
    let initial_hash_equal=e.state_hash()==r.state_hash();assert!(initial_hash_equal);
    let old=e.log().len();let original=submit(e,A::Answer{choice_id:choice.id,selection:selection.clone()},*seq);
    let resumed=submit(&mut r,A::Answer{choice_id:choice.id,selection:selection.clone()},*seq);*seq+=1;
    let mut after=r.save();after.auth_generation=e.save().auth_generation;
    let full=after==e.save();let hash=e.state_hash()==r.state_hash();let logs=&e.log()[old..]==r.log();let outcome=format!("{original:?}")==format!("{resumed:?}");
    let row=json!({"prompt":choice.spec.prompt,"observer":choice.observer,"kind":format!("{:?}",choice.spec.kind),"min":choice.spec.min,"max":choice.spec.max,"allow_cancel":choice.spec.allow_cancel,"options":choice.spec.options.iter().map(|x|format!("{x:?}")).collect::<Vec<_>>(),"selection":selection.iter().map(|x|format!("{x:?}")).collect::<Vec<_>>(),"archive_file":file,"archive_bytes":bytes.len(),"decode_restore_ok":true,"full_snapshot_before_equal_except_auth_generation":true,"full_snapshot_after_equal_except_auth_generation":full,"state_hash_equal":hash,"outcome_equal":outcome,"new_logs_equal":logs,"outcome":format!("{original:?}"),"resumed_outcome":format!("{resumed:?}"),"log_delta":e.log()[old..].iter().map(|x|format!("{x:?}")).collect::<Vec<_>>(),"after":state(e)});
    trace.push(row);assert!(full&&hash&&logs&&outcome,"restore continuation differs {case}");
}
fn run(case:&str,v:&VerifiedLockedContent,base:&Path)->Value{
    let mut e=start_locked_engine_with_original_presentation(2026111428,8,LockedStartMode::Prototype,v,None).unwrap();
    e.state_core_mut().skills.clear();for s in 0..8{let p=e.state_core_mut().player_mut(s);p.hand.clear();p.equips.clear();p.judge.clear();p.hp=8;p.max_hp=8;}
    e.state_core_mut().discard.clear();e.state_core_mut().table.clear();e.state_core_mut().deck=(0..e.catalog().cards.len()as u32).rev().collect();e.debug_set_turn(0,Phase::Play,1);
    let skill=if case.starts_with("buxu"){"th_buxu"}else if case.starts_with("zhengjing"){"th_zhengjing"}else{"th_yeyan"};
    if !case.ends_with("no-skill"){grant(&mut e,skill);}
    let cost=card(&e,CardKind::Sha);let alternative=card(&e,CardKind::Shan);
    let suits=[Suit::Spade,Suit::Club,Suit::Heart,Suit::Diamond];let four:Vec<_>=suits.iter().map(|s|e.catalog().cards.iter().position(|c|c.suit==*s).unwrap()as u32).collect();
    if case.starts_with("buxu")&&!case.ends_with("empty"){take(&mut e,cost);take(&mut e,alternative);let tao=card(&e,CardKind::Tao);take(&mut e,tao);}
    if case.starts_with("buxu")&&case.ends_with("full"){
        let kinds=[CardKind::Duel,CardKind::Tao,CardKind::Nullify,CardKind::Wuzhong,CardKind::Le,CardKind::Weapon];
        let mut classics=Vec::new();for k in kinds{let id=card(&e,k);e.state_core_mut().deck.retain(|x|*x!=id);e.state_core_mut().player_mut(0).hand.retain(|x|*x!=id);classics.push(id);}
        e.state_core_mut().vm_aux.piles.insert((0,"th_liujing".to_owned()),classics);flag(&mut e,"th_chengye_mask",63);flag(&mut e,"th_chengye_liujing_count",6);
    }
    if case.starts_with("yeyan"){
        if case.ends_with("no-four"){take(&mut e,cost);}else{for id in &four{take(&mut e,*id);}}
        if case.ends_with("used"){flag(&mut e,"th_yeyan_used",1);let d=e.catalog().skill_by_name(skill).unwrap();e.state_core_mut().skills.get_mut(&(0,d)).unwrap().once_used=true;}
    }
    let initial=state(&e);let log_start=e.log().len();let mut trace=Vec::new();let mut activations=Vec::new();let mut seq=1;
    let rounds=if case=="buxu-twice"{2}else{1};
    for iteration in 0..rounds{
        let d=e.catalog().skill_by_name(skill).unwrap();let result=submit(&mut e,A::ActivateSkill{skill:d},seq);seq+=1;activations.push(format!("{result:?}"));
        let mut steps=0;while let Some(open)=e.open_choice().cloned(){
            steps+=1;assert!(steps<15,"unexpected loop {case}");
            if case=="buxu-invalid"&&open.spec.prompt=="th_buxu:choose_chamber"{answer(&mut e,v,base,case,vec![O::Tag(6)],&mut seq,&mut trace);}
            if case=="yeyan-no-four"&&open.spec.prompt=="th_yeyan:choose_mode"{answer(&mut e,v,base,case,vec![O::Tag(1)],&mut seq,&mut trace);}
            let selection=match open.spec.prompt{
                "th_buxu:discard_cards"=>if case=="buxu-cancel"{vec![]}else{open.spec.options.iter().take(open.spec.min).copied().collect()},
                "th_buxu:choose_chamber"=>vec![O::Tag(if case=="buxu-equip"||iteration==1{5}else if case=="buxu-nullify"{2}else{1})],
                "th_zhengjing:choose_count"=>if case=="zhengjing-cancel"{vec![]}else{vec![O::Tag(if case=="zhengjing-zero"{0}else if case=="zhengjing-three-target"{3}else{1})]},
                "th_zhengjing:choose_target"=>if case=="zhengjing-cancel"{vec![]}else{vec![O::Player(if case=="zhengjing-three-target"{1}else{0})]},
                "th_yeyan:choose_mode"=>if case=="yeyan-cancel"{vec![]}else{vec![O::Tag(u8::from(case.contains("great")))]},
                "th_yeyan:choose_targets"=>if case=="yeyan-small-three"{vec![O::Player(1),O::Player(2),O::Player(3)]}else if case=="yeyan-great-split"{vec![O::Player(3),O::Player(1)]}else{vec![O::Player(1)]},
                "th_yeyan:discard_cards"=>four.iter().map(|c|O::Card(*c)).collect(),
                p=>panic!("unexpected prompt {p} {case}")};
            answer(&mut e,v,base,case,selection,&mut seq,&mut trace);
        }
    }
    let final_archive=base.join(format!("{case}-final.ngsa"));fs::write(&final_archive,encode_archive(&e.save())).unwrap();
    json!({"case":case,"skill":skill,"fixture_kind":"formal locked-package bridge and attach; debug initial state only","initial":initial,"cards":{"cost":cost,"alternative":alternative,"four_suits":four},"activations":activations,"choices":trace,"final":state(&e),"final_archive":final_archive,"log":e.log()[log_start..].iter().map(|x|format!("{x:?}")).collect::<Vec<_>>(),"settled":e.open_choice().is_none()})
}
fn main(){
    let root=PathBuf::from(std::env::var("NONAME_REPO_ROOT").unwrap());let output=PathBuf::from(std::env::var("L28_OUTPUT").unwrap());fs::create_dir_all(&output).unwrap();
    let package=root.join("packages/extensions/thunder-upstream-liuli");let raw=fs::read_to_string(package.join("package.npack.json")).unwrap();let mut assets=BTreeMap::new();assets.insert("assets/config.json".to_owned(),fs::read(package.join("assets/config.json")).unwrap());
    let frozen=validate_and_freeze_raw_package(raw,assets).unwrap();fs::write(output.join("formal-frozen-package.json"),serde_json::to_vec_pretty(&frozen).unwrap()).unwrap();let verified=prepare_locked_content_with_bridge(vec![frozen]).unwrap();
    for case in ["buxu-basic","buxu-equip","buxu-nullify","buxu-cancel","buxu-empty","buxu-full","buxu-twice","buxu-invalid","zhengjing-one-self","zhengjing-three-target","zhengjing-zero","zhengjing-cancel","zhengjing-no-skill","yeyan-small-one","yeyan-small-three","yeyan-great-one","yeyan-great-split","yeyan-cancel","yeyan-used","yeyan-no-four"]{
        let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||run(case,&verified,&output)));
        match result{Ok(row)=>println!("{row}"),Err(err)=>{let msg=err.downcast_ref::<String>().cloned().or_else(||err.downcast_ref::<&str>().map(|s|s.to_string()));println!("{}",json!({"case":case,"probe_error":msg}));}}
    }
}
