//! Headless CLI for engine debug and golden hash dumps.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;
use std::env;

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
            // Optional trailing `--ruleset <name>` selects the mode (identity-standard,
            // guozhan, single). M4: 单挑 mode is playable through this entry.
            let mut ruleset = "identity-standard".to_string();
            if args.next().as_deref() == Some("--ruleset") {
                if let Some(name) = args.next() {
                    ruleset = name;
                }
            }
            run_match(players, seed, max_steps, &ruleset);
        }
        "full-demo" => {
            // End-to-end match demo with turn-by-turn output.
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
            // Phase 14: pin MatchManifest at StartGame and print hash + chain status.
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            print_manifest(players, seed);
        }
        "replay-export" => {
            // Phase 16: export replay file, re-import, verify manifest + chain.
            // Optional output path: replay-export [players] [seed] [out.json]
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            let out_path = args.next();
            run_replay_export(players, seed, out_path.as_deref());
        }
        "replay-import" => {
            // Import a replay JSON file, reconstruct engine, verify chain.
            let path = args.next().unwrap_or_else(|| {
                eprintln!("usage: engine-cli replay-import <file.json>");
                std::process::exit(1);
            });
            run_replay_import(&path);
        }
        "skill-vm" => {
            // Load official jianxiong.skill (or path), start, grant, damage → draw via VM
            let path = args
                .next()
                .unwrap_or_else(|| "content/standard/jianxiong.skill".into());
            run_skill_vm(&path);
        }
        "load-hash" => {
            // Start with optional package JSON → print content lock + start hash
            let pkg_path = args.next();
            let players: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
            let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(42);
            run_load_hash(pkg_path.as_deref(), players, seed);
        }
        "mod-demo" => {
            // Load mashu+qicao package, grant, print distance/range queries
            run_mod_demo();
        }
        other => {
            eprintln!("unknown command: {other}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        "engine-cli — noname-next headless core\n\n\
         Usage:\n\
           engine-cli start [players] [seed]   Start match and print state summary\n\
           engine-cli hash [players] [seed]    Print canonical state hash only\n\
           engine-cli manifest [players] [seed]\n\
                                               Pin MatchManifest and print its hash + chain status\n\
           engine-cli replay-export [players] [seed]\n\
                                               Export replay file, verify manifest + chain\n\
           engine-cli replay-import <file.json>\n\
                                               Import replay JSON, reconstruct engine, verify chain\n\
           engine-cli demo                     Start + end phases, print revisions\n\
           engine-cli play                     Start, enter play phase, try first hand card\n\
           engine-cli match [players] [seed] [max_steps]\n\
                                               Run full identity match with auto-players\n\
           engine-cli full-demo [players] [seed] [max_steps]\n\
                                               End-to-end match demo with outcome summary\n\
           engine-cli skill-vm [skill.dsl]     Load DSL package, damage → VM draw demo\n\
           engine-cli mod-demo                 Load mashu+qicao, print query results\n\
           engine-cli load-hash [pkg.json] [players] [seed]\n\
                                               Optional package load then start hash\n\
           engine-cli help\n"
    );
}

fn run_mod_demo() {
    use engine_api::SkillDefId;
    use engine_core::effect::run_effects;
    use engine_core::query::{query, QueryContext};
    use engine_core::transaction::Transaction;
    use engine_core::{BOOTSTRAP_MASHU, BOOTSTRAP_QICAO};
    use rule_api::{EffectRequest, QueryRequest, QueryResponse};

    let mut src = String::new();
    src.push_str(
        &std::fs::read_to_string("content/standard/mashu.skill").unwrap_or_else(|e| {
            eprintln!("read mashu: {e}");
            std::process::exit(2);
        }),
    );
    src.push('\n');
    src.push_str(
        &std::fs::read_to_string("content/standard/qicao.skill").unwrap_or_else(|e| {
            eprintln!("read qicao: {e}");
            std::process::exit(2);
        }),
    );
    let pkg = content_compiler::compile_source(&src).unwrap_or_else(|e| {
        eprintln!("compile: {e}");
        std::process::exit(2);
    });
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 11,
            ruleset: "identity-standard".into(),
        },
    });
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: p1,
                skill_def: SkillDefId::new(BOOTSTRAP_MASHU),
            },
            EffectRequest::AddSkill {
                player: p1,
                skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
            },
        ],
    )
    .unwrap();
    let mut ctx = QueryContext::new();
    let dist = query(
        tx.state(),
        &QueryRequest::Distance { from: p1, to: p2 },
        &mut ctx,
    );
    let range = query(
        tx.state(),
        &QueryRequest::AttackRange { player: p1 },
        &mut ctx,
    );
    println!("packages={}", eng.content().package_count());
    println!("distance_1_to_2={dist:?}");
    println!("attack_range_1={range:?}");
    match (dist, range) {
        (QueryResponse::Int(d), QueryResponse::Int(r)) if r >= 2 && d <= 1 => {
            println!("ok=true");
        }
        _ => {
            println!("ok=false");
            std::process::exit(3);
        }
    }
}

fn run_skill_vm(path: &str) {
    // Skill-VM smoke: compile the DSL package, drive one card-backed damage into the owning
    // player, and verify the skill's authoritative contract. The standard 奸雄 (jianxiong)
    // contract is "after taking card-backed damage, optionally gain that physical carrier"
    // (upstream standard/skill.js `damageEnd` + `gain(trigger.cards)`). A cardless DealDamage
    // intentionally opens no choice — see original_standard_jianxiong.rs for the full golden.
    use engine_api::{CardDefId, CardId, ChoiceId, SkillDefId, Suit};
    use engine_core::command::apply_command;
    use engine_core::effect::{run_effects, EffectError};
    use engine_core::state::{Card, CardKind, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("read {path}: {e}");
        std::process::exit(2);
    });
    let pkg = content_compiler::compile_source(&source).unwrap_or_else(|e| {
        eprintln!("compile: {e}");
        std::process::exit(2);
    });
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap_or_else(|e| {
        eprintln!("load: {e}");
        std::process::exit(2);
    });
    eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 7,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = PlayerId::new(1);
    let opponent = PlayerId::new(2);
    let mut tx = Transaction::begin(eng.state().clone());
    for player in [owner, opponent] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(1),
        }],
    )
    .unwrap();

    // Place the damage carrier (a Sha) in the processing zone, then deal card-backed damage.
    // jianxiong fires only when a physical carrier is in processing at damageEnd.
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .expect("processing zone exists");
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .expect("processing zone exists")
        .cards
        .push(card);

    let before = tx.state().hand_count(owner);
    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: opponent,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    let pending = tx.state().has_pending_choice();
    let state = tx.commit().state;

    println!("package={}", path);
    println!("content_lock={}", hex32(&eng.state().ruleset.logic_hash));
    println!("hand_before={before}");
    println!(
        "damage_opened_choice={}",
        matches!(damage, Err(EffectError::ChoiceRequested))
    );

    // Card-backed damage must open the optional gain; resolving it (selection 1) gains the carrier.
    let ok = if matches!(damage, Err(EffectError::ChoiceRequested)) && pending {
        let (next, resolved) = apply_command(
            state.clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        if !matches!(resolved.result, CommandResult::Accepted { .. }) {
            println!("ok=false");
            false
        } else {
            let gained = next.hand_cards(owner).contains(&card);
            let after = next.hand_count(owner);
            println!("hand_after={after}");
            println!("gained_carrier={gained}");
            println!("ok={}", gained && after == before + 1);
            gained && after == before + 1
        }
    } else {
        // No carrier-backed damage → no gain contract; this smoke only exercises the gain path.
        println!("ok=false");
        false
    };

    if !ok {
        std::process::exit(3);
    }
}

fn run_load_hash(pkg_path: Option<&str>, players: u8, seed: u64) {
    let mut eng = Engine::new();
    if let Some(p) = pkg_path {
        let data = std::fs::read(p).unwrap_or_else(|e| {
            eprintln!("read package: {e}");
            std::process::exit(2);
        });
        let pkg = content_schema::ContentPackage::from_json_bytes(&data).unwrap_or_else(|e| {
            eprintln!("parse package: {e}");
            std::process::exit(2);
        });
        eng.load_package(&pkg).unwrap_or_else(|e| {
            eprintln!("load: {e}");
            std::process::exit(2);
        });
        println!(
            "packages={} content_hash={}",
            eng.content().package_count(),
            hex32(&eng.state().ruleset.logic_hash)
        );
    }
    eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: players,
            rng_seed: seed,
            ruleset: "bootstrap".into(),
        },
    });
    println!("hash={}", hex32(&eng.state_hash()));
}

fn run_match(players: u8, seed: u64, max_steps: u32, ruleset: &str) {
    use engine_core::{run_identity_match, MatchConfig};
    match run_identity_match(MatchConfig {
        player_count: players,
        rng_seed: seed,
        max_steps,
        ruleset: ruleset.to_string(),
    }) {
        Ok(r) => {
            println!("outcome={:?}", r.outcome);
            println!("steps={}", r.steps);
            println!("revisions={}", r.revisions);
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
    use engine_core::{run_identity_match, MatchConfig};

    println!("╔══════════════════════════════════════╗");
    println!("║   无名杀 Next — 端到端对局演示      ║");
    println!("╚══════════════════════════════════════╝");
    println!();
    println!("配置: {players} 人身份局, seed={seed}, max_steps={max_steps}");
    println!();

    // Run the match.
    let report = match run_identity_match(MatchConfig {
        player_count: players,
        rng_seed: seed,
        max_steps,
        ruleset: "identity-standard".into(),
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("match error: {e}");
            std::process::exit(2);
        }
    };

    // Print outcome.
    println!("━━━ 终局结果 ━━━");
    println!("  outcome:    {:?}", report.outcome);
    println!("  steps:      {}", report.steps);
    println!("  revisions:  {}", report.revisions);
    println!("  capped:     {}", report.capped);
    println!("  state_hash: {}", hex32(&report.state_hash));
    println!();

    // Print manifest info.
    println!("━━━ 对局 Manifest ━━━");
    println!("  build:      {}", engine_api::ENGINE_BUILD_ID);
    println!();

    // Print player summary.
    println!("━━━ 玩家概况 ━━━");
    // We can't access the final state from MatchReport, but we can show
    // the outcome interpretation.
    match &report.outcome {
        engine_api::MatchOutcome::RebelWin => {
            println!("  反贼胜利！主公阵亡。");
        }
        engine_api::MatchOutcome::LoyalistWin => {
            println!("  忠臣/主公胜利！所有反贼和内奸已阵亡。");
        }
        engine_api::MatchOutcome::RenegadeWin { player } => {
            println!("  内奸 P{} 获得最终胜利！", player.raw());
        }
        engine_api::MatchOutcome::Draw => {
            println!("  平局。");
        }
        engine_api::MatchOutcome::Aborted { reason_code } => {
            println!("  对局中止 (reason={reason_code})。");
        }
        engine_api::MatchOutcome::InProgress => {
            println!("  对局未结束（达到步数上限）。");
        }
        engine_api::MatchOutcome::GroupWin { group } => {
            println!("  {group} 势力胜利！（国战模式）");
        }
        engine_api::MatchOutcome::SingleWin { player } => {
            println!("  P{} 单挑获胜！（单挑模式）", player.raw());
        }
    }
    println!();

    if report.capped {
        println!(
            "⚠ 对局在 {} 步后未自然结束（可能需要更多步数）。",
            report.steps
        );
    } else {
        println!(
            "✓ 对局自然结束，用时 {} 步 / {} 次状态变更。",
            report.steps, report.revisions
        );
    }

    // Exit code: 0 = natural finish, 3 = capped.
    if report.capped {
        std::process::exit(3);
    }
}

fn run_start(players: u8, seed: u64) {
    let mut eng = Engine::new();
    let result = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: players,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    match result {
        CommandResult::Accepted {
            new_revision,
            state_hash,
        } => {
            println!("revision={new_revision}");
            println!("hash={}", hex32(&state_hash));
            println!("players={}", eng.state().players.live_count());
            println!("cards={}", eng.state().cards.live_count());
            println!(
                "hands={}",
                eng.state()
                    .players
                    .iter_live()
                    .map(|p| format!("{}:{}", p.id.raw(), eng.state().hand_count(p.id)))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            println!("build={}", engine_api::ENGINE_BUILD_ID);
        }
        CommandResult::Rejected { reason } => {
            eprintln!("rejected: {reason:?}");
            std::process::exit(2);
        }
    }
}

fn print_hash(players: u8, seed: u64) {
    let mut eng = Engine::new();
    let _ = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: players,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    println!("{}", hex32(&eng.state_hash()));
}

fn print_manifest(players: u8, seed: u64) {
    // Phase 14: demonstrate MatchManifest freeze + hash-chain integrity.
    let mut eng = Engine::new();
    let _ = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: players,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let m = match eng.manifest() {
        Some(m) => m,
        None => {
            eprintln!("manifest not pinned");
            std::process::exit(2);
        }
    };
    let mh = eng.manifest_hash();
    println!("manifest_version={}", m.manifest_version);
    println!("semantics={}", m.engine_semantics_id.label);
    println!("semantics_revision={}", m.engine_semantics_id.revision);
    println!("rule_ir={}", m.rule_ir_version.raw());
    println!("vm_abi={}", m.vm_abi_version.raw());
    println!("ruleset={}", m.ruleset.name);
    println!("content_lock={}", hex32(&m.content_lock_hash.0));
    println!("rng_seed={}", m.rng_spec.initial_seed);
    println!("effect_budget={}", m.budget_profile.effect_budget);
    println!("trigger_budget={}", m.budget_profile.trigger_budget);
    println!("players={}", m.match_config.player_count);
    println!("manifest_hash={}", hex32(&mh.0));
    let log = eng.replay_log();
    println!("records={}", log.records.len());
    println!("commands={}", log.commands.len());
    match log.verify_chain() {
        Ok(()) => println!("chain=ok"),
        Err(i) => {
            println!("chain=BROKEN@{i}");
            std::process::exit(3);
        }
    }
}

fn run_replay_export(players: u8, seed: u64, out_path: Option<&str>) {
    // Phase 16: export a replay file, re-import, verify manifest + chain.
    let mut eng = Engine::new();
    let _ = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: players,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let pinned = eng.manifest_hash();
    let bytes = eng.export_replay().unwrap_or_else(|e| {
        eprintln!("export replay: {e}");
        std::process::exit(2);
    });
    println!("replay_bytes={}", bytes.len());
    println!("export_manifest_hash={}", hex32(&pinned.0));

    // Optionally write to disk.
    if let Some(path) = out_path {
        std::fs::write(path, &bytes).unwrap_or_else(|e| {
            eprintln!("write {path}: {e}");
            std::process::exit(2);
        });
        println!("wrote={path}");
    }

    let imported = Engine::import_replay(&bytes, Some(pinned)).unwrap_or_else(|e| {
        eprintln!("import replay: {e}");
        std::process::exit(2);
    });
    println!("imported_state_hash={}", hex32(&imported.state_hash()));
    println!(
        "imported_manifest_hash={}",
        hex32(&imported.manifest_hash().0)
    );
    imported.replay_log().verify_chain().unwrap_or_else(|i| {
        eprintln!("chain broken at {i}");
        std::process::exit(3);
    });
    if imported.state_hash() == eng.state_hash() && imported.manifest_hash() == pinned {
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

    // Try capsule format first; fall back to plain replay log.
    let text = String::from_utf8_lossy(&bytes);
    if text.contains("\"capsule_schema\"") {
        // ReplayCapsule format.
        let (eng, capsule) = Engine::open_capsule(&bytes).unwrap_or_else(|e| {
            eprintln!("import failed: {e}");
            std::process::exit(2);
        });
        println!("format:   capsule");
        println!("records:  {}", eng.replay_log().records.len());
        println!("commands: {}", eng.replay_log().commands.len());
        println!("revision: {}", eng.state().revision);
        println!("hash:     {}", hex32(&eng.state_hash()));
        println!("manifest: {}", hex32(&eng.manifest_hash().0));
        eng.replay_log().verify_chain().unwrap_or_else(|i| {
            println!("chain=BROKEN@{i}");
            std::process::exit(3);
        });
        println!("chain=ok");
        capsule.verify().unwrap_or_else(|e| {
            println!("capsule=INVALID: {e}");
            std::process::exit(3);
        });
        println!("capsule=ok");
    } else {
        // Plain ReplayLog format.
        let log = engine_core::ReplayLog::from_bytes(&bytes).unwrap_or_else(|e| {
            eprintln!("parse replay log: {e}");
            std::process::exit(2);
        });
        println!("format:   replay_log");
        println!("records:  {}", log.records.len());
        println!("commands: {}", log.commands.len());
        println!("manifest: {}", hex32(&log.manifest_hash().0));
        log.verify_chain().unwrap_or_else(|i| {
            println!("chain=BROKEN@{i}");
            std::process::exit(3);
        });
        println!("chain=ok");
        // Replay commands to reconstruct.
        let commands: Vec<Command> = log.commands.iter().map(|c| c.command.clone()).collect();
        let eng = Engine::replay_from(&commands).unwrap_or_else(|e| {
            eprintln!("replay: {e}");
            std::process::exit(2);
        });
        println!("revision: {}", eng.state().revision);
        println!("hash:     {}", hex32(&eng.state_hash()));
    }
}

fn run_demo() {
    let mut eng = Engine::new();
    let cmds = [
        Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2026,
                ruleset: "bootstrap".into(),
            },
        },
        Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        },
        Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        },
    ];
    for (i, c) in cmds.into_iter().enumerate() {
        let r = eng.submit(c);
        match r {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } => {
                println!(
                    "step={i} revision={new_revision} hash={}",
                    hex32(&state_hash)
                );
            }
            CommandResult::Rejected { reason } => {
                eprintln!("step={i} rejected: {reason:?}");
                std::process::exit(2);
            }
        }
    }
    println!("final_hash={}", hex32(&eng.state_hash()));
}

fn run_play_demo() {
    use engine_core::{CardKind, Phase};

    let mut eng = Engine::new();
    let start = Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 42,
            ruleset: "bootstrap".into(),
        },
    };
    match eng.submit(start) {
        CommandResult::Accepted {
            new_revision,
            state_hash,
        } => println!(
            "start revision={new_revision} hash={} hand={}",
            hex32(&state_hash),
            eng.state().hand_count(eng.state().turn.current)
        ),
        CommandResult::Rejected { reason } => {
            eprintln!("start rejected: {reason:?}");
            std::process::exit(2);
        }
    }
    let actor = eng.state().turn.current;
    // Draw -> Play
    let _ = eng.submit(Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    assert_eq!(eng.state().turn.phase, Phase::Play);
    let hand = eng.state().hand_cards(actor);
    println!(
        "play phase hand={:?}",
        hand.iter().map(|c| c.raw()).collect::<Vec<_>>()
    );
    for card in hand {
        let kind = eng.state().cards.get(card).map(|c| c.kind);
        let targets = match kind {
            Some(CardKind::Sha) => eng
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|p| *p != actor)
                .collect::<Vec<_>>(),
            _ => vec![],
        };
        let r = eng.submit(Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::PlayCard {
                card,
                targets: targets.clone(),
                card_targets: vec![],
            },
        });
        match r {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } => {
                println!(
                    "played card={} kind={:?} targets={:?} revision={new_revision} hash={} choice={}",
                    card.raw(),
                    kind,
                    targets.iter().map(|p| p.raw()).collect::<Vec<_>>(),
                    hex32(&state_hash),
                    eng.state().has_pending_choice()
                );
                break;
            }
            CommandResult::Rejected { reason } => {
                println!("skip card={} kind={:?}: {reason:?}", card.raw(), kind);
            }
        }
    }
    println!("final_hash={}", hex32(&eng.state_hash()));
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// temporary debug - will remove
#[allow(dead_code)]
fn debug_stuck() {
    use engine_core::choose_command;
    let mut eng = Engine::new();
    eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 6,
            rng_seed: 1006,
            ruleset: "identity-standard".into(),
        },
    });
    let mut steps = 1u32;
    while !eng.state().outcome.is_finished() && steps < 1500 {
        let pending = eng.state().has_pending_choice();
        let cont = eng.state().resolution.continuation_kind;
        let phase = eng.state().turn.phase;
        let actor = eng.state().turn.current;
        let cmd = choose_command(&eng);
        let kind = format!("{:?}", cmd.kind);
        match eng.submit(cmd) {
            CommandResult::Accepted { .. } => steps += 1,
            CommandResult::Rejected { reason } => {
                eprintln!("FAIL step={steps} rev={} phase={:?} actor={} pending_before={pending} cont={cont:?}",
                    eng.state().revision, phase, actor.raw());
                eprintln!("  cmd={kind}");
                eprintln!("  reason={reason:?}");
                eprintln!(
                    "  pending_after={} cont={:?} obs={}",
                    eng.state().has_pending_choice(),
                    eng.state().resolution.continuation_kind,
                    eng.state().resolution.pending_observer
                );
                for p in eng.state().players.iter_live() {
                    eprintln!(
                        "  p{} alive={} dying={} hp={} hand={}",
                        p.id.raw(),
                        p.alive,
                        p.dying,
                        p.hp,
                        eng.state().hand_count(p.id)
                    );
                }
                // dump payload
                eprintln!(
                    "  payload={:?}",
                    eng.state().resolution.continuation_payload
                );
                return;
            }
        }
    }
    eprintln!("finished steps={steps}");
}
