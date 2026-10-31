use engine_native::{ai::BoundedPlanner, Engine};
use std::{collections::BTreeMap, sync::Mutex};
static COUNTS: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());
pub fn record(engine: &Engine, seat: u8, planner: &BoundedPlanner) {
 let thread = std::thread::current(); let name = thread.name().unwrap_or("unnamed");
 let selected = ["nc10_ai_bounds::", "nc10_ai_information_boundary::", "policy_v2_drives_all_seats", "nc07_discard_duplicate_probe::", "nc12_ai_strength_baseline::", "nc12_swd_authoritative_stability::"];
 if !selected.iter().any(|prefix| name.contains(prefix)) { return; }
 let mut counts = COUNTS.lock().unwrap(); let count = counts.entry(name.to_owned()).or_default();
 if *count >= 32 { return; } let index = *count; *count += 1;
 let input = engine.ai_session(seat).input();
 let class = match &input.choice { Some(engine_native::ChoiceView::Own{..})=>"Own",Some(engine_native::ChoiceView::Other{..})=>"Other",None=>"None" };
 let path = std::path::Path::new("/tmp/nc10-own-36992565/evidence/domain-cases"); std::fs::create_dir_all(path).unwrap();
 let base = format!("{}-{}", name.replace("::","--"), index);
 let archive = engine_native::archive::encode_archive(&engine.save()); std::fs::write(path.join(format!("{base}.ngsa")),archive).unwrap();
 let row=serde_json::json!({"test":name,"index":index,"seat":seat,"class":class,"input":format!("{input:?}"),"pre_events":format!("{:?}",engine.event_view_for(seat)),"nodes":planner.node_budget,"deadline_ms":planner.deadline_ms,"variant":planner.policy_variant,"depth":planner.lookahead_depth,"note":"Current330 public domain sample; not an exact historical private NoCert location or a privacy proof"});
 std::fs::write(path.join(format!("{base}.json")),serde_json::to_string_pretty(&row).unwrap()).unwrap();
}
