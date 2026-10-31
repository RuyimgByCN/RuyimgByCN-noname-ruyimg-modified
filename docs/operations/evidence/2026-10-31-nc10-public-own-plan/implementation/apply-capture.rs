use engine_native::CommandOutcome;
use std::{fs::OpenOptions,io::Write,sync::Mutex};
static LOCK: Mutex<()> = Mutex::new(());
pub fn record(outcome: CommandOutcome) -> CommandOutcome {
 let _lock=LOCK.lock().unwrap();let thread=std::thread::current();let name=thread.name().unwrap_or("unknown");
 let row=serde_json::json!({"test":name,"outcome":format!("{outcome:?}")});
 let mut file=OpenOptions::new().create(true).append(true).open("/tmp/nc10-own-36992565/evidence/actual-positive-applies.jsonl").unwrap();writeln!(file,"{}",row).unwrap();outcome
}
