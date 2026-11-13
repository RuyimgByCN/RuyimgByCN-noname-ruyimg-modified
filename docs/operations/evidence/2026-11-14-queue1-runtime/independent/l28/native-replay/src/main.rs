use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine_with_original_presentation,
    FrozenPackage,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::{ChoiceOption, GameAction, GameCommand};
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

fn selection(items: &Value) -> Vec<ChoiceOption> {
    items.as_array().unwrap().iter().map(|item| {
        let s = item.as_str().unwrap();
        let open = s.find('(').unwrap();
        let n: u32 = s[open + 1..s.len() - 1].parse().unwrap();
        match &s[..open] {
            "Tag" => ChoiceOption::Tag(n as u8),
            "Card" => ChoiceOption::Card(n),
            "Player" => ChoiceOption::Player(n as u8),
            other => panic!("unrecognized selection {other}"),
        }
    }).collect()
}

fn main() {
    let input = PathBuf::from(env::var("L28_REVIEW_INPUT").unwrap());
    let output = PathBuf::from(env::var("L28_REPLAY_OUTPUT").unwrap());
    fs::create_dir_all(&output).unwrap();
    let frozen: FrozenPackage = serde_json::from_slice(
        &fs::read(input.join("native/round2/formal-frozen-package.json")).unwrap(),
    ).unwrap();
    let verified = prepare_locked_content_with_bridge(vec![frozen]).unwrap();
    let rows: Vec<Value> = fs::read_to_string(input.join("native/round2.jsonl")).unwrap()
        .lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    let mut pending_count = 0;
    let mut failed = 0;
    for row in &rows {
        let case = row["case"].as_str().unwrap();
        let choices = row["choices"].as_array().unwrap();
        for (index, choice) in choices.iter().enumerate() {
            pending_count += 1;
            let path = PathBuf::from(choice["archive_file"].as_str().unwrap());
            let bytes = fs::read(&path).unwrap();
            let decoded = decode_archive(&bytes).unwrap();
            let restored = restore_locked_engine_with_original_presentation(&decoded, &verified, None);
            let mut engine = match restored {
                Ok(e) => e,
                Err(e) => { failed += 1; println!("{}", json!({"case":case,"index":index,"restore_error":format!("{e:?}")})); continue; }
            };
            let mut normalized_before = engine.save();
            normalized_before.auth_generation = decoded.auth_generation;
            let before_equal = normalized_before == decoded;
            let open = engine.open_choice().unwrap().clone();
            let spec_equal = open.spec.prompt == choice["prompt"].as_str().unwrap()
                && format!("{:?}",open.spec.kind) == choice["kind"].as_str().unwrap()
                && open.observer as u64 == choice["observer"].as_u64().unwrap()
                && open.spec.min as u64 == choice["min"].as_u64().unwrap()
                && open.spec.max as u64 == choice["max"].as_u64().unwrap()
                && open.spec.allow_cancel == choice["allow_cancel"].as_bool().unwrap()
                && open.spec.options.iter().map(|x|format!("{x:?}")).collect::<Vec<_>>()
                   == choice["options"].as_array().unwrap().iter().map(|x|x.as_str().unwrap().to_string()).collect::<Vec<_>>();
            let dedup_id = index as u64 + 2 + u64::from(case == "buxu-twice" && index >= 2);
            let outcome = engine.submit(GameCommand {
                actor: open.observer,
                expected_revision: engine.revision(),
                dedup_id,
                action: GameAction::Answer { choice_id: open.id, selection: selection(&choice["selection"]) },
            });
            let hash_equal = format!("{:?}",engine.state_hash()) == choice["after"]["state_hash"].as_str().unwrap();
            let outcome_equal = format!("{outcome:?}") == choice["outcome"].as_str().unwrap();
            let logs: Vec<String> = engine.log().iter().map(|x|format!("{x:?}")).collect();
            let logs_equal = logs == choice["log_delta"].as_array().unwrap().iter().map(|x|x.as_str().unwrap().to_string()).collect::<Vec<_>>();
            let mut expected_after = None;
            if !(case == "buxu-twice" && index == 1) {
                let target = if index + 1 < choices.len() { choices[index + 1]["archive_file"].as_str().unwrap() }
                    else { row["final_archive"].as_str().unwrap() };
                let snapshot = decode_archive(&fs::read(target).unwrap()).unwrap();
                let mut normalized_after = engine.save();
                normalized_after.auth_generation = snapshot.auth_generation;
                expected_after = Some(normalized_after == snapshot);
            }
            let before_file = output.join(format!("{case}-{index:02}-before-snapshot.txt"));
            let after_file = output.join(format!("{case}-{index:02}-after-snapshot.txt"));
            fs::write(&before_file,format!("{decoded:#?}\n")).unwrap();
            fs::write(&after_file,format!("{:#?}\n",engine.save())).unwrap();
            fs::write(output.join(format!("{case}-{index:02}-after.ngsa")), encode_archive(&engine.save())).unwrap();
            let passed = before_equal && spec_equal && hash_equal && outcome_equal && logs_equal && expected_after != Some(false);
            if !passed { failed += 1; }
            println!("{}",json!({"kind":"pending","case":case,"index":index,"archive":path,
                "dedup_id":dedup_id,"before_full_snapshot_equal_except_auth":before_equal,"choice_spec_equal":spec_equal,
                "after_state_hash_equal":hash_equal,"outcome_equal":outcome_equal,"new_logs_equal":logs_equal,
                "after_full_snapshot_equal_to_next_or_final_except_auth":expected_after,
                "after_full_snapshot_skipped_reason":if expected_after.is_none(){Some("collector performed second ActivateSkill before the next pending archive")}else{None},
                "outcome":format!("{outcome:?}"),"log":logs,"before_snapshot":before_file,"after_snapshot":after_file,"passed":passed}));
        }
        let bytes = fs::read(row["final_archive"].as_str().unwrap()).unwrap();
        let decoded = decode_archive(&bytes).unwrap();
        let engine = restore_locked_engine_with_original_presentation(&decoded, &verified, None).unwrap();
        let mut normalized = engine.save();
        normalized.auth_generation = decoded.auth_generation;
        let equal = normalized == decoded;
        let hash = format!("{:?}",engine.state_hash()) == row["final"]["state_hash"].as_str().unwrap();
        let no_choice = engine.open_choice().is_none();
        let passed = equal && hash && no_choice;
        if !passed { failed += 1; }
        fs::write(output.join(format!("{case}-final-snapshot.txt")),format!("{decoded:#?}\n")).unwrap();
        println!("{}",json!({"kind":"final","case":case,"full_snapshot_equal_except_auth":equal,"state_hash_equal":hash,"no_pending_choice":no_choice,"passed":passed}));
    }
    println!("{}",json!({"kind":"summary","pending":pending_count,"final":rows.len(),"failed":failed}));
    if failed > 0 { std::process::exit(1); }
}
