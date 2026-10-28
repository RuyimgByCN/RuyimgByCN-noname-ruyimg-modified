//! Diagnostic only: same machine, same source and workload, debug build.
use engine_native::{ai::redact_opponent_hands, event::Phase, Engine};
use std::time::Instant;

#[test]
fn redaction_timing() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json")).unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    for with_trap in [false, true] {
        let mut engine = Engine::new_with_extensions(3900, 4, &[&pack]).unwrap();
        engine.debug_set_turn(0, Phase::Play, 1);
        engine.debug_grant(1, "sxianjing_gain");
        if with_trap {
            let card = engine.state_core_mut().deck.pop().unwrap();
            engine.state_core_mut().players[1].sxianjing_traps.push(card);
            for owner in [1usize, 2, 0] {
                let index = engine.state_core().deck.iter().position(|id| {
                    let suit = engine.catalog().cards[*id as usize].suit;
                    engine.state_core().players[owner].sxianjing_traps.iter().all(|other| engine.catalog().cards[*other as usize].suit != suit)
                }).unwrap();
                let extra = engine.state_core_mut().deck.remove(index);
                engine.state_core_mut().players[owner].sxianjing_traps.push(extra);
            }
        }
        let mut probe = engine.clone();
        redact_opponent_hands(&mut probe, 0);
        let restorable = Engine::restore_with_extensions(&probe.save(), &[&pack]).is_ok();
        for _ in 0..200 {
            let mut sim = engine.clone();
            redact_opponent_hands(&mut sim, 0);
            std::hint::black_box(sim);
        }
        let mut samples = Vec::with_capacity(2000);
        for _ in 0..2000 {
            let mut sim = engine.clone();
            let start = Instant::now();
            redact_opponent_hands(&mut sim, 0);
            samples.push(start.elapsed().as_nanos());
            std::hint::black_box(sim);
        }
        samples.sort_unstable();
        eprintln!("multi_trap={with_trap} restorable={restorable} samples={} p50={}ns p95={}ns p99={}ns", samples.len(), samples[1000], samples[1900], samples[1980]);
    }
}
