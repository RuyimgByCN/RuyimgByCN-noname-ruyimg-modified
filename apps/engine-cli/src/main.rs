//! Headless CLI for engine-native debug, replay, and golden hash dumps.

use std::env;

use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, validate_and_freeze_raw_package,
    LockedStartMode,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive, ARCHIVE_FORMAT_VERSION};
use engine_native::catalog::CardKind;
use engine_native::content::standard::STANDARD_RULESET_VERSION;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Outcome, Phase,
};
use rule_vm::{CollectingHost, ExecResult, HostContext, RuleVm, VerifiedModule};

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".into());
    match cmd.as_str() {
        "help" | "-h" | "--help" => print_help(),
        "start" => {
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            run_start(players, seed);
        }
        "demo" => run_demo(),
        "play" => run_play_demo(),
        "match" => {
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            let max_steps: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(2000);
            let mut ruleset = "identity-standard".to_string();
            if args.next().as_deref() == Some("--ruleset") {
                if let Some(name) = args.next() {
                    ruleset = name;
                }
            }
            run_match(players, seed, max_steps, &ruleset);
        }
        "full-demo" => {
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            let max_steps: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(3000);
            run_full_demo(players, seed, max_steps);
        }
        "hash" => {
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            print_hash(players, seed);
        }
        "manifest" => {
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            print_manifest(players, seed);
        }
        "replay-export" => {
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            let out_path = args.next();
            run_replay_export(players, seed, out_path.as_deref());
        }
        "replay-import" => {
            let path = args.next().unwrap_or_else(|| {
                eprintln!("usage: engine-cli replay-import <file.ngsa>");
                std::process::exit(1);
            });
            run_replay_import(&path);
        }
        "skill-vm" => {
            let path = args
                .next()
                .unwrap_or_else(|| "content/standard/jianxiong.skill".into());
            run_skill_vm(&path);
        }
        "load-hash" => {
            let pkg_path = args.next();
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            run_load_hash(pkg_path.as_deref(), players, seed);
        }
        "mod-demo" => run_mod_demo(),
        other => {
            eprintln!("unknown command: {other}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        "engine-cli — noname-next headless native engine\n\n\
         Usage:\n\
           engine-cli start [players] [seed]   Start match and print state summary\n\
           engine-cli hash [players] [seed]    Print canonical state hash only\n\
           engine-cli manifest [players] [seed]\n\
                                               Print ruleset/archive contract + hash chain\n\
           engine-cli replay-export [players] [seed] [out.ngsa]\n\
                                               Export NGSA archive and verify roundtrip\n\
           engine-cli replay-import <file.ngsa>\n\
                                               Import NGSA archive and print state hash\n\
           engine-cli demo                     Start + end phases, print revisions\n\
           engine-cli play                     Start, enter play phase, try first hand card\n\
           engine-cli match [players] [seed] [max_steps]\n\
                                               Run full auto-match with BoundedPlanner\n\
           engine-cli full-demo [players] [seed] [max_steps]\n\
                                               End-to-end match demo with outcome summary\n\
           engine-cli skill-vm [skill.dsl]     Compile DSL package and verify VM execution\n\
           engine-cli mod-demo                 Grant mashu+qicao, print distance/range queries\n\
           engine-cli load-hash [pkg.npack.json] [players] [seed]\n\
                                               Optional package load then start hash\n\
           engine-cli help\n"
    );
}

fn create_engine_for_ruleset(players: u8, seed: u64, ruleset: &str) -> Result<Engine, String> {
    let seats = players as usize;
    match ruleset {
        "identity-standard" | "identity" => Engine::try_new_identity(seed, seats),
        "guozhan" | "kingdoms" => {
            if seats < 2 {
                return Err(format!("kingdoms requires >= 2 seats, got {seats}"));
            }
            Ok(Engine::new_kingdoms(seed, seats))
        }
        "single" => Ok(Engine::new_single(seed)),
        "prototype" | "bootstrap" => Ok(Engine::new(seed, seats.max(1))),
        other => Err(format!("unknown ruleset: {other}")),
    }
}

fn step_auto(eng: &mut Engine, planner: &BoundedPlanner, dedup: &mut u64) -> Result<(), String> {
    let seat = eng
        .open_choice()
        .map(|c| c.observer)
        .unwrap_or_else(|| eng.turn().current);
    let plan = eng.plan_action(seat, planner);
    *dedup += 1;
    let out = eng.submit(GameCommand {
        actor: seat,
        expected_revision: plan.revision,
        dedup_id: *dedup,
        action: plan.action,
    });
    match out {
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => Ok(()),
        CommandOutcome::Rejected(reason) => {
            // Fallback to NextPhase if a planned play was rejected.
            if eng.open_choice().is_none() {
                *dedup += 1;
                let fallback = eng.submit(GameCommand {
                    actor: eng.turn().current,
                    expected_revision: eng.revision(),
                    dedup_id: *dedup,
                    action: GameAction::NextPhase,
                });
                match fallback {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => Ok(()),
                    other => Err(format!("step rejected ({reason:?}); fallback failed: {other:?}")),
                }
            } else {
                Err(format!("choice step rejected: {reason:?}"))
            }
        }
        CommandOutcome::Suspended => {
            while eng.is_suspended() {
                match eng.resume() {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => return Ok(()),
                    CommandOutcome::Suspended => continue,
                    CommandOutcome::Rejected(r) => return Err(format!("resume rejected: {r:?}")),
                }
            }
            Ok(())
        }
    }
}

struct MatchSummary {
    outcome: Option<Outcome>,
    steps: u32,
    revision: u64,
    capped: bool,
    state_hash: [u8; 32],
}

fn execute_auto_match(
    players: u8,
    seed: u64,
    max_steps: u32,
    ruleset: &str,
) -> Result<MatchSummary, String> {
    let mut eng = create_engine_for_ruleset(players, seed, ruleset)?;
    let planner = BoundedPlanner::new(64);
    let mut dedup = 0u64;
    let mut steps = 0u32;
    while eng.outcome().is_none() && steps < max_steps {
        step_auto(&mut eng, &planner, &mut dedup)?;
        steps += 1;
    }
    Ok(MatchSummary {
        outcome: eng.outcome().cloned(),
        steps,
        revision: eng.revision(),
        capped: eng.outcome().is_none(),
        state_hash: eng.state_hash(),
    })
}

fn run_mod_demo() {
    let mut eng = Engine::new(11, 4);
    eng.debug_grant(0, "马术");
    eng.debug_grant(0, "奇才");
    let dist = eng.state_core().computed_distance(eng.catalog(), 0, 1);
    let range = eng.state_core().attack_range(eng.catalog(), 0);
    println!("skills={}", eng.catalog().skills.len());
    println!("distance_0_to_1={dist}");
    println!("attack_range_0={range}");
    if dist <= 1 && range >= 1 {
        println!("ok=true");
    } else {
        println!("ok=false");
        std::process::exit(3);
    }
}

fn run_skill_vm(path: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("read {path}: {e}");
        std::process::exit(2);
    });
    let pkg = content_compiler::compile_source(&source).unwrap_or_else(|e| {
        eprintln!("compile: {e}");
        std::process::exit(2);
    });
    let module = pkg.modules.first().unwrap_or_else(|| {
        eprintln!("no VM modules in {path}");
        std::process::exit(2);
    });
    let verified = VerifiedModule::verify(module.clone()).unwrap_or_else(|e| {
        eprintln!("verify {}: {e}", module.name);
        std::process::exit(2);
    });
    let mut host = CollectingHost {
        ctx: HostContext {
            skill_id: 1,
            self_id: 1,
            event_player: 1,
            event_source: 2,
            event_card: 1,
            event_amount: 1,
        },
        ..Default::default()
    };
    let mut vm = RuleVm::new();
    let mut res = vm.execute(&verified, &mut host, 10_000).unwrap_or_else(|e| {
        eprintln!("vm execute: {e}");
        std::process::exit(3);
    });
    if let ExecResult::Paused {
        continuation,
        options,
    } = res
    {
        let chosen = options.last().copied().unwrap_or(1);
        res = vm
            .resume(&verified, &mut host, continuation, chosen)
            .unwrap_or_else(|e| {
                eprintln!("vm resume: {e}");
                std::process::exit(3);
            });
    }
    println!("package={path}");
    println!("logic_hash={}", hex32(&pkg.manifest.logic_hash));
    println!("module={}", module.name);
    println!("effects={}", host.effects.len());
    let ok = matches!(res, ExecResult::Halted { .. });
    println!("ok={ok}");
    if !ok {
        std::process::exit(3);
    }
}

fn run_load_hash(pkg_path: Option<&str>, players: u8, seed: u64) {
    let eng = if let Some(path_str) = pkg_path {
        let path = std::path::Path::new(path_str);
        let raw = std::fs::read_to_string(path)
            .unwrap_or_else(|e| {
                eprintln!("read {path_str}: {e}");
                std::process::exit(2);
            })
            .replace('\r', "");
        let pkg = content_schema::ContentPackage::from_json_bytes(raw.as_bytes()).unwrap_or_else(|e| {
            eprintln!("parse package {path_str}: {e}");
            std::process::exit(2);
        });
        let mut assets = std::collections::BTreeMap::new();
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        for rel in pkg.asset_paths.values() {
            let full = parent.join(rel);
            let raw_bytes = std::fs::read(&full).unwrap_or_else(|e| {
                eprintln!("read asset {}: {e}", full.display());
                std::process::exit(2);
            });
            let bytes = if rel.ends_with(".json") || rel.ends_with(".skill") {
                raw_bytes.into_iter().filter(|&b| b != b'\r').collect()
            } else {
                raw_bytes
            };
            assets.insert(rel.clone(), bytes);
        }
        let frozen = validate_and_freeze_raw_package(raw, assets).unwrap_or_else(|e| {
            eprintln!("validate package: {e}");
            std::process::exit(2);
        });
        let verified = prepare_locked_content_with_bridge(vec![frozen]).unwrap_or_else(|e| {
            eprintln!("prepare locked content: {e}");
            std::process::exit(2);
        });
        println!("content_hash={}", verified.lock().lock_hash);
        start_locked_engine(seed, players as usize, LockedStartMode::Prototype, &verified)
            .unwrap_or_else(|e| {
                eprintln!("start locked engine: {e}");
                std::process::exit(2);
            })
    } else {
        Engine::new(seed, players as usize)
    };
    println!("hash={}", hex32(&eng.state_hash()));
}

fn run_match(players: u8, seed: u64, max_steps: u32, ruleset: &str) {
    match execute_auto_match(players, seed, max_steps, ruleset) {
        Ok(r) => {
            println!("outcome={:?}", r.outcome);
            println!("steps={}", r.steps);
            println!("revisions={}", r.revision);
            println!("capped={}", r.capped);
            println!("hash={}", hex32(&r.state_hash));
            if r.capped {
                std::process::exit(3);
            }
        }
        Err(e) => {
            eprintln!("match error: {e}");
            std::process::exit(2);
        }
    }
}

fn run_full_demo(players: u8, seed: u64, max_steps: u32) {
    println!("╔══════════════════════════════════════╗");
    println!("║   无名杀 Next — 原生端到端对局演示  ║");
    println!("╚══════════════════════════════════════╝");
    println!();
    println!("配置: {players} 人身份局, seed={seed}, max_steps={max_steps}");
    println!();

    let report = match execute_auto_match(players, seed, max_steps, "identity-standard") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("match error: {e}");
            std::process::exit(2);
        }
    };

    println!("━━━ 终局结果 ━━━");
    println!("  outcome:    {:?}", report.outcome);
    println!("  steps:      {}", report.steps);
    println!("  revisions:  {}", report.revision);
    println!("  capped:     {}", report.capped);
    println!("  state_hash: {}", hex32(&report.state_hash));
    println!();
    println!("━━━ 对局版本 ━━━");
    println!("  build:      {}", engine_api::ENGINE_BUILD_ID);
    println!("  ruleset:    v{STANDARD_RULESET_VERSION}");
    println!("  archive:    NGSA v{ARCHIVE_FORMAT_VERSION}");
    if report.capped {
        std::process::exit(3);
    }
}

fn run_start(players: u8, seed: u64) {
    let eng = if (4..=8).contains(&players) {
        Engine::new_identity(seed, players as usize)
    } else {
        Engine::new(seed, (players as usize).max(1))
    };
    let core = eng.state_core();
    println!("revision={}", eng.revision());
    println!("hash={}", hex32(&eng.state_hash()));
    println!("players={}", core.players.iter().filter(|p| p.alive).count());
    println!("cards={}", eng.catalog().cards.len());
    println!(
        "hands={}",
        (0..core.players.len())
            .map(|p| format!("{p}:{}", core.players[p].hand.len()))
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("build={}", engine_api::ENGINE_BUILD_ID);
}

fn print_hash(players: u8, seed: u64) {
    let eng = Engine::new(seed, (players as usize).max(1));
    println!("{}", hex32(&eng.state_hash()));
}

fn print_manifest(players: u8, seed: u64) {
    let eng = Engine::new(seed, (players as usize).max(1));
    let bytes = encode_archive(&eng.save());
    let manifest_hash = *blake3::hash(&bytes).as_bytes();
    let roundtrip_ok = decode_archive(&bytes)
        .ok()
        .and_then(|snap| Engine::restore(&snap).ok())
        .map(|restored| restored.state_hash() == eng.state_hash())
        .unwrap_or(false);
    println!("ruleset_version={STANDARD_RULESET_VERSION}");
    println!("archive_format_version={ARCHIVE_FORMAT_VERSION}");
    println!("players={players}");
    println!("seed={seed}");
    println!("manifest_hash={}", hex32(&manifest_hash));
    println!("chain={}", if roundtrip_ok { "ok" } else { "BROKEN" });
    if !roundtrip_ok {
        std::process::exit(3);
    }
}

fn run_replay_export(players: u8, seed: u64, out_path: Option<&str>) {
    let eng = Engine::new(seed, (players as usize).max(1));
    let bytes = encode_archive(&eng.save());
    let archive_hash = *blake3::hash(&bytes).as_bytes();
    println!("archive_bytes={}", bytes.len());
    println!("export_archive_hash={}", hex32(&archive_hash));

    if let Some(path) = out_path {
        std::fs::write(path, &bytes).unwrap_or_else(|e| {
            eprintln!("write {path}: {e}");
            std::process::exit(2);
        });
        println!("wrote={path}");
    }

    let snap = decode_archive(&bytes).unwrap_or_else(|e| {
        eprintln!("decode archive: {e:?}");
        std::process::exit(2);
    });
    let restored = Engine::restore(&snap).unwrap_or_else(|e| {
        eprintln!("restore archive: {e:?}");
        std::process::exit(2);
    });
    println!("imported_state_hash={}", hex32(&restored.state_hash()));
    if restored.state_hash() == eng.state_hash() {
        println!("replay_roundtrip=ok");
    } else {
        println!("replay_roundtrip=MISMATCH");
        std::process::exit(3);
    }
}

fn run_replay_import(path: &str) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| {
        eprintln!("read {path}: {e}");
        std::process::exit(2);
    });
    let snap = decode_archive(&bytes).unwrap_or_else(|e| {
        eprintln!("decode NGSA archive failed: {e:?}");
        std::process::exit(2);
    });
    let eng = Engine::restore(&snap).unwrap_or_else(|e| {
        eprintln!("restore NGSA archive failed: {e:?}");
        std::process::exit(2);
    });
    println!("format:   NGSA v{ARCHIVE_FORMAT_VERSION}");
    println!("ruleset:  v{}", eng.ruleset_version());
    println!("revision: {}", eng.revision());
    println!("hash:     {}", hex32(&eng.state_hash()));
}

fn run_demo() {
    let mut eng = Engine::new(2026, 4);
    let mut dedup = 0u64;
    for i in 0..2 {
        let actor = eng.turn().current;
        dedup += 1;
        let out = eng.submit(GameCommand {
            actor,
            expected_revision: eng.revision(),
            dedup_id: dedup,
            action: GameAction::NextPhase,
        });
        match out {
            CommandOutcome::Accepted { state_hash } => {
                println!("step={i} revision={} hash={}", eng.revision(), hex32(&state_hash));
            }
            CommandOutcome::Pending { choice } => {
                println!(
                    "step={i} pending choice={} observer={}",
                    choice.id, choice.observer
                );
                dedup += 1;
                let sel = choice.spec.options.iter().take(choice.spec.min.max(1)).copied().collect();
                let _ = eng.submit(GameCommand {
                    actor: choice.observer,
                    expected_revision: eng.revision(),
                    dedup_id: dedup,
                    action: GameAction::Answer {
                        choice_id: choice.id,
                        selection: sel,
                    },
                });
            }
            CommandOutcome::Rejected(reason) => {
                eprintln!("step={i} rejected: {reason:?}");
                std::process::exit(2);
            }
            CommandOutcome::Suspended => {}
        }
    }
    println!("final_hash={}", hex32(&eng.state_hash()));
}

fn run_play_demo() {
    let mut eng = Engine::new(42, 2);
    println!(
        "start revision={} hash={} hand={}",
        eng.revision(),
        hex32(&eng.state_hash()),
        eng.debug_hand(eng.turn().current).len()
    );
    let mut dedup = 0u64;
    for _ in 0..8 {
        if eng.turn().phase == Phase::Play {
            break;
        }
        let actor = eng.turn().current;
        dedup += 1;
        if let CommandOutcome::Pending { choice } = eng.submit(GameCommand {
            actor,
            expected_revision: eng.revision(),
            dedup_id: dedup,
            action: GameAction::NextPhase,
        }) {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .take(choice.spec.min.max(1))
                .copied()
                .collect();
            dedup += 1;
            let _ = eng.submit(GameCommand {
                actor: choice.observer,
                expected_revision: eng.revision(),
                dedup_id: dedup,
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: sel,
                },
            });
        }
    }
    let actor = eng.turn().current;
    let hand = eng.debug_hand(actor);
    println!("play phase hand={hand:?}");
    for card in hand {
        let kind = eng.catalog().card_kind(card);
        let targets = if kind == CardKind::Sha {
            vec![1 - actor]
        } else {
            vec![]
        };
        dedup += 1;
        match eng.submit(GameCommand {
            actor,
            expected_revision: eng.revision(),
            dedup_id: dedup,
            action: GameAction::PlayCard {
                card,
                targets: targets.clone(),
            },
        }) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {
                println!(
                    "played card={card} kind={kind:?} targets={targets:?} revision={} hash={}",
                    eng.revision(),
                    hex32(&eng.state_hash())
                );
                break;
            }
            CommandOutcome::Rejected(reason) => {
                println!("skip card={card} kind={kind:?}: {reason:?}");
            }
            CommandOutcome::Suspended => {}
        }
    }
    println!("final_hash={}", hex32(&eng.state_hash()));
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_hash_and_archive_roundtrip() {
        let a = Engine::new(42, 4);
        let b = Engine::new(42, 4);
        assert_eq!(a.state_hash(), b.state_hash());
        let bytes = encode_archive(&a.save());
        let snap = decode_archive(&bytes).unwrap();
        let restored = Engine::restore(&snap).unwrap();
        assert_eq!(restored.state_hash(), a.state_hash());
    }

    #[test]
    fn native_auto_match_runs() {
        let summary = execute_auto_match(4, 42, 200, "identity-standard").unwrap();
        assert!(summary.steps > 0);
    }
}
