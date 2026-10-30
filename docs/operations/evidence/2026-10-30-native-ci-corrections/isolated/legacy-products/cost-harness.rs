use engine_core::{Engine, GameState, canonical_state_hash};
use engine_core::content::{ContentRegistry, ensure_standard_package};
use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use std::{alloc::{GlobalAlloc, Layout, System}, sync::atomic::{AtomicU64, Ordering}, time::Instant, hint::black_box};
static OPS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
struct Alloc;
unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { OPS.fetch_add(1,Ordering::Relaxed); BYTES.fetch_add(layout.size() as u64,Ordering::Relaxed); System.alloc(layout) }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) { System.dealloc(p,layout) }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 { OPS.fetch_add(1,Ordering::Relaxed); BYTES.fetch_add(size as u64,Ordering::Relaxed); System.realloc(p,layout,size) }
}
#[global_allocator] static ALLOC: Alloc = Alloc;
fn start(seed:u64, players:u8, ruleset:&str) -> Command {
    Command { actor:PlayerId::NONE, expected_revision:ExpectedRevision(0), idempotency_key:IdempotencyKey(0), kind:CommandKind::StartGame {player_count:players,rng_seed:seed,ruleset:ruleset.into()} }
}
fn measure<T>(f:impl FnOnce()->T)->(T,u64,u64,u64) {
    let a=OPS.load(Ordering::Relaxed); let b=BYTES.load(Ordering::Relaxed); let t=Instant::now();
    let value=black_box(f()); let ns=t.elapsed().as_nanos() as u64;
    (value,ns,OPS.load(Ordering::Relaxed)-a,BYTES.load(Ordering::Relaxed)-b)
}
fn profile() {
    let mut warm=Engine::new(); assert!(matches!(warm.submit(start(42,4,"identity-standard")),CommandResult::Accepted{..}));
    for i in 0..200 {
        let (mut e,n,a,b)=measure(Engine::new); println!("{{\"sample\":{i},\"phase\":\"ctor\",\"ns\":{n},\"allocs\":{a},\"bytes\":{b}}}");
        let (r,n,a,b)=measure(||e.submit(start(42,4,"identity-standard"))); assert!(matches!(r,CommandResult::Accepted{..})); println!("{{\"sample\":{i},\"phase\":\"submit\",\"ns\":{n},\"allocs\":{a},\"bytes\":{b}}}");
        let (_,n,a,b)=measure(||canonical_state_hash(e.state())); println!("{{\"sample\":{i},\"phase\":\"state_hash\",\"ns\":{n},\"allocs\":{a},\"bytes\":{b}}}");
        let mut p=Engine::new(); let (_,n,a,b)=measure(||p.load_standard_package().unwrap()); println!("{{\"sample\":{i},\"phase\":\"load_standard\",\"ns\":{n},\"allocs\":{a},\"bytes\":{b}}}");
        let (_,n,a,b)=measure(||p.submit(start(42,4,"identity-standard"))); println!("{{\"sample\":{i},\"phase\":\"submit_preloaded\",\"ns\":{n},\"allocs\":{a},\"bytes\":{b}}}");
        let mut registry=ContentRegistry::new(); let (_,n,a,b)=measure(||ensure_standard_package(&mut registry)); println!("{{\"sample\":{i},\"phase\":\"uncached_registry\",\"ns\":{n},\"allocs\":{a},\"bytes\":{b}}}");
    }
}
fn traces() {
    for ruleset in ["bootstrap","identity-standard","single","guozhan"] { for (players,seed) in [(2,1),(4,42),(8,99)] {
        if ruleset=="single" && players!=2 {continue;}
        for dynamic in [false,true] {
            let mut e=Engine::new();
            if dynamic { let pkg=content_compiler::compile_source_named("private-overlay","skill jianxiong(1) locked { trigger damage owner_is_target { effect draw_cards(self, 2); } }\nskill private_extra(4000000000) { modifier distance_from add 1; }").unwrap(); e.load_package(&pkg).unwrap(); }
            let result=e.submit(start(seed,players,ruleset)); assert!(matches!(result,CommandResult::Accepted{..}));
            for step in 0..16 {
                println!("{}",serde_json::json!({"ruleset":ruleset,"players":players,"seed":seed,"dynamic":dynamic,"step":step,"state":e.state(),"hash":e.state_hash(),"events":e.last_events(),"choice":e.last_choice(),"manifest":e.manifest(),"replay":e.export_replay().unwrap()}));
                let kind=if let Some(choice)=e.last_choice() { CommandKind::ResolveChoice{choice_id:choice.choice_id,selections:choice.options.iter().take(choice.min_count.max(1) as usize).map(|o|o.token).collect(),cancelled:choice.options.is_empty() && choice.allow_cancel} } else { CommandKind::EndPhase };
                let actor=e.last_choice().map(|c|c.observer).unwrap_or(e.state().turn.current);
                let _=e.submit(Command{actor,expected_revision:ExpectedRevision(e.state().revision),idempotency_key:IdempotencyKey(step+1),kind});
            }
        }
    }}
    black_box(GameState::empty());
}
fn main() { match std::env::args().nth(1).as_deref() { Some("traces")=>traces(),Some("loop")=>{for _ in 0..100_000 {let mut e=Engine::new();black_box(e.submit(start(42,4,"identity-standard")));}},_=>profile() } }
