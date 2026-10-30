use engine_native::{Engine, MachineSnapshot, GameAction, GameCommand, CommandOutcome};
use engine_native::ai::redact_opponent_hands;
use engine_native_wasm::{engine_from_archive, hex_encode};
use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};

fn machines(machine: &MachineSnapshot, out: &mut Vec<Value>) {
    use MachineSnapshot::*;
    match machine {
        Rvm { owner, committing, commit_state, following, commit_pick, paused, .. } => {
            out.push(json!({"owner":owner,"committing":committing,"commit_state":commit_state,
                "following_count":following.len(),"commit_pick":format!("{commit_pick:?}"),"paused":paused.is_some()}));
            for machine in following { machines(machine,out); }
        }
        OptionalGate { inner, .. } | NullifyWindow { effect:inner, .. } | Aoe { inner, .. }
        | Chain { inner, .. } | Jijiang { inner, .. } | LiJian { inner, .. } => {
            if let Some(inner)=inner { machines(inner,out); }
        }
        _=>{}
    }
}
fn payload(e: &Engine) -> Value {
    let snapshot=e.save(); let mut states=vec![];
    for park in &snapshot.resolution.parks { machines(&park.machine,&mut states); }
    json!({"rvm":states,"resolution":format!("{:?}",snapshot.resolution),
        "hands":e.state_core().players.iter().map(|p|p.hand.clone()).collect::<Vec<_>>(),
        "equips":e.state_core().players.iter().map(|p|p.equips.clone()).collect::<Vec<_>>()})
}
fn answer(e: &mut Engine) -> Result<String,String> {
    let c=e.open_choice().unwrap().clone();
    let selection=c.spec.options.iter().take(c.spec.min).copied().collect();
    let out=e.submit(GameCommand{actor:c.observer,expected_revision:e.revision(),dedup_id:e.save().last_dedup+1,
        action:GameAction::Answer{choice_id:c.id,selection}});
    if matches!(out,CommandOutcome::Rejected(_)){Err(format!("{out:?}"))}else{Ok(format!("{out:?}"))}
}
fn panic_message(v: Box<dyn std::any::Any+Send>)->String {
    v.downcast_ref::<String>().cloned().or_else(||v.downcast_ref::<&str>().map(|s|s.to_string())).unwrap_or("panic".into())
}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    let data:Value=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let mut records=Vec::new(); let mut failures=0;
    for scenario in data["scenarios"].as_array().unwrap(){
        let name=scenario["name"].as_str().unwrap();
        let packs=scenario["packs_json"].as_str().unwrap();
        let mut checkpoints=vec![("initial".to_string(),scenario["initial"].as_str().unwrap().to_string())];
        checkpoints.extend(scenario["steps"].as_array().unwrap().iter().enumerate().map(|(i,s)|(format!("step{i}"),s["archive"].as_str().unwrap().to_string())));
        for (checkpoint,archive) in checkpoints {
            let original=engine_from_archive(&archive,packs).unwrap();
            let Some(choice)=original.open_choice() else {continue};
            let observer=choice.observer;
            let prompt=choice.spec.prompt;
            let before=payload(&original);
            let control=catch_unwind(AssertUnwindSafe(||{let mut e=engine_from_archive(&archive,packs).unwrap();answer(&mut e)}));
            let control=match control{Ok(Ok(v))=>json!({"valid":true,"outcome":v}),Ok(Err(e))=>json!({"valid":false,"error":e}),Err(e)=>json!({"valid":false,"panic":panic_message(e)})};
            let mut sampled=original.clone();
            let result=catch_unwind(AssertUnwindSafe(||{
                redact_opponent_hands(&mut sampled,observer);
                let saved=sampled.save();
                let hex=hex_encode(&engine_native::archive::encode_archive(&saved));
                let mut restored=engine_from_archive(&hex,packs).map_err(|e|format!("restore: {e}"))?;
                answer(&mut restored).map(|out|json!({"valid":true,"outcome":out,"after_hash":hex_encode(&restored.state_hash())}))
            }));
            let result=match result{Ok(Ok(v))=>v,Ok(Err(e))=>json!({"valid":false,"error":e}),Err(e)=>json!({"valid":false,"panic":panic_message(e)})};
            let fail=control["valid"]==true && result["valid"]!=true;
            if fail {failures+=1;}
            let record=json!({"scenario":name,"checkpoint":checkpoint,"prompt":prompt,"observer":observer,"control":control,
                "sampled_result":result,"regression":fail,"before":before,"after_redaction":payload(&sampled),
                "source_unchanged":hex_encode(&engine_native::archive::encode_archive(&original.save()))==hex_encode(&engine_native::archive::encode_archive(&engine_from_archive(&archive,packs).unwrap().save()))});
            println!("{}",json!({"scenario":name,"checkpoint":checkpoint,"prompt":prompt,"observer":observer,"regression":fail,"sampled_result":record["sampled_result"]}));
            records.push(record);
        }
    }
    let output=json!({"current_matrix":args[1],"checked":records.len(),"failures":failures,"records":records});
    std::fs::write(&args[2],serde_json::to_vec_pretty(&output).unwrap()).unwrap();
    assert_eq!(failures,0,"real J2 pending-window helper regressions");
}
