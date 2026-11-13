use engine_native::archive::{decode_archive, encode_archive};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn number_after(text: &str, key: &str) -> u64 {
    text.split_once(key)
        .unwrap()
        .1
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap()
}

fn selection(text: &str) -> Vec<ChoiceOption> {
    let raw = text.split_once("selection: [").unwrap().1.split(']').next().unwrap();
    raw.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            if part.starts_with("Card(") {
                ChoiceOption::Card(number_after(part, "Card(") as u32)
            } else if part.starts_with("Player(") {
                ChoiceOption::Player(number_after(part, "Player(") as u8)
            } else if part.starts_with("Tag(") {
                ChoiceOption::Tag(number_after(part, "Tag(") as u8)
            } else {
                match part {
                    "Pass" => ChoiceOption::Pass,
                    "Confirm" => ChoiceOption::Confirm,
                    _ => panic!("unknown recorded option {part}"),
                }
            }
        })
        .collect()
}

fn command(text: &str) -> GameCommand {
    let action = if text.contains("action: Answer {") {
        GameAction::Answer {
            choice_id: number_after(text, "choice_id: "),
            selection: selection(text),
        }
    } else if text.contains("action: ActivateSkill {") {
        GameAction::ActivateSkill {
            skill: number_after(text, "skill: ") as u16,
        }
    } else if text.contains("action: PlayCard {") {
        let targets = text.split_once("targets: [").unwrap().1.split(']').next().unwrap();
        GameAction::PlayCard {
            card: number_after(text, "card: ") as u32,
            targets: targets.split(',').map(str::trim).filter(|p| !p.is_empty()).map(|p| p.parse().unwrap()).collect(),
        }
    } else {
        panic!("unhandled recorded command {text}");
    };
    GameCommand {
        actor: number_after(text, "actor: ") as u8,
        expected_revision: number_after(text, "expected_revision: "),
        dedup_id: number_after(text, "dedup_id: "),
        action,
    }
}

fn main() {
    let original = PathBuf::from(std::env::var("U08_ORIGINAL").unwrap());
    let repo = PathBuf::from(std::env::var("U08_REPO").unwrap());
    let out = PathBuf::from(std::env::var("U08_REVIEW_OUT").unwrap());
    fs::create_dir(&out).unwrap();
    let input = fs::read_to_string(repo.join("packages/extensions/hero-kill-upstream-noname/assets/cards.json")).unwrap().replace("\r\n", "\n");
    let mut pack = content_native_loader::bridge_pack(&input).unwrap().pack;
    content_native_loader::attach_formal_package_upstream_rules("upstream-noname", "hero-kill", &mut pack, &[]);
    let runs: Vec<Value> = serde_json::from_slice(&fs::read(original.join("native-round2/native-results.json")).unwrap()).unwrap();
    let mut results = Vec::new();
    let inspect = Engine::new_with_extensions(202611140008, 8, &[&pack]).unwrap();
    let registration = json!({
        "namespace":"upstream-noname", "package_id":"hero-kill", "host_skills":pack.host_skills.len(),
        "vm_skills":pack.vm_skills.len(), "pack_cards":pack.cards.len(),
        "content_lock":engine_native::content::extension::content_lock_id(&[&pack]),
        "bound_ruleset":inspect.save().ruleset_version,
        "skills":(["yixin", "bolehuiyan", "bolehuiyan2", "bolehuiyan3", "guifu"].iter().map(|name|json!({"name":name,"id":inspect.catalog().skill_by_name(name)})).collect::<Vec<_>>())
    });
    fs::write(out.join("formal-registration.json"), serde_json::to_vec_pretty(&registration).unwrap()).unwrap();
    for run in runs {
        let case = run["case"].as_str().unwrap();
        for mode in ["direct", "restored"] {
            let data = &run[mode];
            let commands = data["commands"].as_array().unwrap();
            let answer_indices = commands.iter().enumerate().filter(|(_, cmd)|cmd["request"].as_str().unwrap().contains("action: Answer {")).map(|(i, _)| i).collect::<Vec<_>>();
            let pending = data["pending_archive_checks"].as_array().unwrap();
            assert_eq!(answer_indices.len(), pending.len());
            let final_bytes = fs::read(original.join("native-round2").join(format!("{case}-{mode}-final.ngsa"))).unwrap();
            let expected_final = decode_archive(&final_bytes).unwrap();
            for (checkpoint, start) in pending.iter().zip(answer_indices) {
                let input_file = checkpoint["file"].as_str().unwrap();
                let bytes = fs::read(original.join("native-round2").join(input_file)).unwrap();
                let snapshot = decode_archive(&bytes).unwrap();
                let mut engine = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
                let restored = engine.save();
                let mut normalized = restored.clone();
                normalized.auth_generation = snapshot.auth_generation;
                let choice = engine.open_choice().unwrap();
                let preceding_choice = &commands[start - 1]["after"]["choice"];
                let choice_matches = json!({"id":choice.id,"observer":choice.observer,"prompt":choice.spec.prompt,
                    "kind":format!("{:?}",choice.spec.kind),"min":choice.spec.min,"max":choice.spec.max,
                    "allow_cancel":choice.spec.allow_cancel,"options":choice.spec.options.iter().map(|o|format!("{o:?}")).collect::<Vec<_>>()}) == *preceding_choice;
                fs::write(out.join(format!("{input_file}-restored-debug.txt")), format!("{restored:#?}")).unwrap();
                let mut steps = Vec::new();
                for (index, expected) in commands.iter().enumerate().skip(start) {
                    let text = expected["request"].as_str().unwrap();
                    let request = command(text);
                    let request_matches = format!("{request:?}") == text;
                    let mark = engine.log().len();
                    let reply = engine.submit(request);
                    let actual_reply = format!("{reply:?}");
                    let actual_logs = engine.log()[mark..].iter().map(|l|format!("{l:?}")).collect::<Vec<_>>();
                    let actual_hash = format!("{:02x?}",engine.state_hash());
                    steps.push(json!({"command_index":index,"request":text,"request_reconstruction_exact":request_matches,
                        "actual_reply":actual_reply,"reply_exact":actual_reply == expected["reply"].as_str().unwrap(),
                        "actual_log_delta":actual_logs,"log_delta_exact":serde_json::to_value(&actual_logs).unwrap()==expected["log_delta"],
                        "actual_state_hash":actual_hash,"state_hash_exact":actual_hash==expected["after"]["hash"].as_str().unwrap()}));
                }
                let final_snapshot = engine.save();
                let mut final_normalized = final_snapshot.clone();
                final_normalized.auth_generation = expected_final.auth_generation;
                let row = json!({"case":case,"mode":mode,"input_file":input_file,"input_decoded_reencoded_bytes_exact":encode_archive(&snapshot)==bytes,
                    "input_full_snapshot_equal_except_auth_generation":normalized==snapshot,
                    "auth_before":snapshot.auth_generation,"auth_after":restored.auth_generation,
                    "restored_choice_equals_recorded_choice":choice_matches,"suffix_commands":steps,
                    "final_full_snapshot_equal_except_auth_generation":final_normalized==expected_final,
                    "final_auth_generation":final_snapshot.auth_generation,"recorded_final_auth_generation":expected_final.auth_generation});
                fs::write(out.join(format!("{input_file}-suffix-final.ngsa")), encode_archive(&final_snapshot)).unwrap();
                fs::write(out.join(format!("{input_file}-suffix-final-debug.txt")), format!("{final_snapshot:#?}")).unwrap();
                results.push(row);
            }
        }
    }
    fs::write(out.join("pending-suffix-results.json"), serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    let all_pass = results.iter().all(|r|r["input_decoded_reencoded_bytes_exact"]==true && r["input_full_snapshot_equal_except_auth_generation"]==true && r["restored_choice_equals_recorded_choice"]==true && r["final_full_snapshot_equal_except_auth_generation"]==true && r["suffix_commands"].as_array().unwrap().iter().all(|s|["request_reconstruction_exact","reply_exact","log_delta_exact","state_hash_exact"].iter().all(|k|s[k]==true)));
    println!("{}", json!({"pending_archive_count":results.len(),"all_pass":all_pass}));
    assert!(all_pass);
}
