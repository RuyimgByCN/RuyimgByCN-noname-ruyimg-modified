//! P2/NC12 保存成本基线：长局推进时 `Engine::save` + archive 编码的耗时与体积。
//!
//! 运行：`cargo run --release -p engine-native --example save_cost`
//! 只测原生内核；不含服务端 WAL/收据/多房间，不把本数据当产品 SLA。

use engine_native::{Engine, GameAction, GameCommand};
use std::time::Instant;

fn percentile(mut values: Vec<f64>, p: f64) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let index = ((values.len() - 1) as f64 * p).round() as usize;
    values[index]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let players: u8 = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(4);
    let steps: usize = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1200);
    let sample_every = 100usize;
    let mut engine = Engine::new(20_261_028, players.into());
    let mut dedup = 1u64;
    let mut saved = Vec::new();
    let mut bytes = Vec::new();
    let mut reached = 0usize;
    for step in 0..steps {
        if engine.outcome().is_some() {
            break;
        }
        if step % sample_every == 0 {
            let mut times = Vec::with_capacity(5);
            let mut last_bytes = 0usize;
            for _ in 0..5 {
                let start = Instant::now();
                let snapshot = engine.save();
                let encoded = engine_native::archive::encode_archive(&snapshot);
                last_bytes = encoded.len();
                times.push(start.elapsed().as_secs_f64() * 1_000_000.0);
            }
            let mut sorted = times.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            saved.push(sorted[2]);
            bytes.push(last_bytes);
            println!(
                "sample step={step} save_p50_us={:.1} archive_bytes={last_bytes} players={players} hp={:?}",
                sorted[2],
                engine.debug_hp(0)
            );
        }
        if let Some(choice) = engine.open_choice() {
            let selection = choice
                .spec
                .options
                .iter()
                .take(choice.spec.min as usize)
                .cloned()
                .collect();
            let actor = choice.observer;
            let choice_id = choice.id;
            let _ = engine.submit(GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: dedup,
                action: GameAction::Answer {
                    choice_id,
                    selection,
                },
            });
        } else {
            let actor = engine.turn().current;
            let _ = engine.submit(GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: dedup,
                action: GameAction::NextPhase,
            });
        }
        dedup += 1;
        reached = step + 1;
    }
    if !saved.is_empty() {
        println!(
            "RESULT players={players} steps={reached} samples={} save_p50_us={:.1} save_p95_us={:.1} archive_bytes_max={} outcome={:?}",
            saved.len(),
            percentile(saved.clone(), 0.5),
            percentile(saved, 0.95),
            bytes.iter().copied().max().unwrap_or(0),
            engine.outcome()
        );
    } else {
        println!("RESULT steps=0 (game ended before first sample)");
    }
}
