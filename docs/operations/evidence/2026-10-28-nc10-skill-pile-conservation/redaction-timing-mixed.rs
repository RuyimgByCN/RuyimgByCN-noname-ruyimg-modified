//! Same-machine diagnostic; not a product performance benchmark.
use engine_native::{ai::redact_opponent_hands, Engine};
use std::hint::black_box;
use std::time::Instant;

fn state(with_piles: bool) -> Engine {
    let mut engine = Engine::new(3910, 4);
    if with_piles {
        let core = engine.state_core_mut();
        core.players[0].zhucheng.push(core.deck.pop().unwrap());
        core.players[1]
            .lianyaohu_store
            .push(core.deck.pop().unwrap());
        core.players[2]
            .sxianjing_traps
            .push(core.deck.pop().unwrap());
        core.players[3].hutian_hold.push(core.deck.pop().unwrap());
        core.vm_aux
            .piles
            .insert((0, "probe".into()), vec![core.deck.pop().unwrap()]);
    }
    engine
}

#[test]
fn same_machine_redaction_latency() {
    for with_piles in [false, true] {
        let engine = state(with_piles);
        let mut verify = engine.clone();
        redact_opponent_hands(&mut verify, 0);
        eprintln!(
            "piles={with_piles} restorable={}",
            Engine::restore(&verify.save()).is_ok()
        );
        for _ in 0..200 {
            let mut sim = engine.clone();
            redact_opponent_hands(&mut sim, 0);
            black_box(sim);
        }
        let mut samples = Vec::with_capacity(2000);
        for _ in 0..2000 {
            let mut sim = engine.clone();
            let start = Instant::now();
            redact_opponent_hands(&mut sim, 0);
            samples.push(start.elapsed().as_nanos());
            black_box(sim);
        }
        samples.sort_unstable();
        eprintln!(
            "piles={with_piles} samples={} p50={}ns p95={}ns p99={}ns",
            samples.len(),
            samples[999],
            samples[1899],
            samples[1979]
        );
    }
}
