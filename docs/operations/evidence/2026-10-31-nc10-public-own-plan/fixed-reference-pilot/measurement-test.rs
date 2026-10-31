//! NC12/N10 AI 强度基线：策略对局长度、终局率与故障率的**统计口径**。
//!
//! 背景：2026-10-19 遮蔽填充池收紧（公开区/在飞区不再进入填充池）后，
//! `ai_planner::policy_v2_drives_all_seats_without_answer_routing_panic`（seed 8211）
//! 步数由 **179 → 484**。本用例给出**同参数下的统计基线**（多 seed 的步数分布与终局率），
//! 供后续判断该变化是否可接受、以及填充策略是否需要重新设计。
//!
//! 说明：本用例**只统计不断言步数**（步数分布是策略面观测，不是规则正确性）；若命中
//! 内核悬空引用故障（`Plan.sim_faults` / `legal_actions::probe_faults()`）则**失败**并打印。

use engine_native::ai::BoundedPlanner;
use engine_native::{Engine, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn swd_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

#[test]
fn nc10_fixed_reference_calibration() {
    use engine_native::{ChoiceView, CommandOutcome};
    let path=std::path::Path::new("/tmp/nc10-fixed-reference-38de1942/evidence");
    let expected:Vec<serde_json::Value>=serde_json::from_str(&std::fs::read_to_string(path.join("calibration-expected.json")).unwrap()).unwrap();
    let planner=BoundedPlanner::new(8).with_deadline_ms(50).with_lookahead(1).with_policy_variant(0);
    let mut engine=swd_engine(45000);let mut own=0;let mut rows=Vec::new();
    for row in expected {
        let seat=row["seat"].as_u64().unwrap() as u8;
        let input=engine.ai_session(seat).input();let before=engine.save();
        let plan=engine_native::ai::measurement_reference_plan(&engine,seat,&planner);
        assert_eq!(engine.save(),before,"switch/planning cannot modify authority/RNG");
        assert_eq!(format!("{input:?}"),row["input"].as_str().unwrap());
        assert_eq!(format!("{plan:?}"),row["plan"].as_str().unwrap());
        if matches!(input.choice,Some(ChoiceView::Own{..})) {
            own+=1;
            let proposal=engine.plan_action(seat,&planner);
            assert_eq!(proposal.nodes_used,0,"reference flag restored after call");
            assert_eq!(engine.save(),before);
        }
        let out=engine.submit(GameCommand{actor:seat,expected_revision:engine.revision(),dedup_id:row["dedup"].as_u64().unwrap(),action:plan.action.clone()});
        assert_eq!(format!("{out:?}"),row["outcome"].as_str().unwrap());
        assert!(matches!(out,CommandOutcome::Accepted{..}|CommandOutcome::Pending{..}));
        rows.push(serde_json::json!({"step":row["step"],"input":format!("{input:?}"),"plan":format!("{plan:?}"),"outcome":format!("{out:?}")}));
    }
    assert!(own>=3);
    std::fs::write(path.join("calibration-actual.json"),serde_json::to_vec_pretty(&serde_json::json!({"rows":rows,"own_calls":own,"all_full_input_plan_outcome_equal":true,"authority_rng_unchanged_by_switch":true})).unwrap()).unwrap();
}

#[test]
fn nc10_fixed_reference_pilot() {
    use engine_native::{ChoiceView,CommandOutcome,Winner};
    use std::{collections::BTreeMap,io::Write};
    let root=std::path::Path::new("/tmp/nc10-fixed-reference-38de1942/evidence");
    let mut trace=std::io::BufWriter::new(std::fs::File::create(root.join("pilot-submits.jsonl")).unwrap());
    let mut games=std::io::BufWriter::new(std::fs::File::create(root.join("pilot-games.jsonl")).unwrap());
    let pack_path=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json");
    let pack=content_native_loader::bridge_pack(&std::fs::read_to_string(pack_path).unwrap()).unwrap().pack;
    let planner=BoundedPlanner::new(8).with_deadline_ms(50).with_lookahead(1).with_policy_variant(0);
    for seed in 46000..46032u64 {
      for challenger in 0..4u8 {
       for arm in ["baseline","candidate"] {
        let mut engine=swd_engine(seed);let mut steps=0usize;let mut counts:BTreeMap<String,u64>=BTreeMap::new();let mut own=0;let mut refcalls=0;let mut proposalcalls=0;let mut simfaults=0u64;
        let game=(|| -> Result<(),String> {
         while engine.outcome().is_none() && steps<1200 {
          let seat=engine.open_choice().map(|c|c.observer).unwrap_or(engine.turn().current);
          let input=engine.ai_session(seat).input();let before=engine.save();let events=engine.event_view_for(seat);
          let reference=arm=="baseline" || seat!=challenger;
          let planned=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| if reference {engine_native::ai::measurement_reference_plan(&engine,seat,&planner)}else{engine.plan_action(seat,&planner)}));
          let plan=match planned {Ok(p)=>p,Err(_)=>{*counts.entry("PlannerPanic".into()).or_default()+=1;return Err("planner panic".into());}};
          assert_eq!(engine.save(),before);
          simfaults+=plan.sim_faults as u64;
          if reference {refcalls+=1;}else{proposalcalls+=1;}
          own+=usize::from(matches!(input.choice,Some(ChoiceView::Own{..})));
          let action=plan.action.clone();let revision=engine.revision();let dedup=steps as u64+1;
          let returned=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||engine.submit(GameCommand{actor:seat,expected_revision:revision,dedup_id:dedup,action})));
          let outcome=match returned {Ok(out)=>out,Err(_)=>{*counts.entry("SubmitPanic".into()).or_default()+=1;return Err("submit panic".into());}};
          let kind=match &outcome {CommandOutcome::Accepted{..}=>"Accepted",CommandOutcome::Pending{..}=>"Pending",CommandOutcome::Rejected(_)=>"Rejected",CommandOutcome::Suspended=>"Suspended"};
          *counts.entry(kind.into()).or_default()+=1;
          writeln!(trace,"{}",serde_json::json!({"seed":seed,"challenger":challenger,"arm":arm,"step":steps,"seat":seat,"reference":reference,"input":format!("{input:?}"),"plan":format!("{plan:?}"),"outcome":format!("{outcome:?}"),"after_revision":engine.revision()})).unwrap();
          steps+=1;
          if matches!(outcome,CommandOutcome::Rejected(_)) || plan.sim_faults>0 {
           std::fs::write(root.join("STOP-before.ngsa"),engine_native::archive::encode_archive(&before)).unwrap();
           std::fs::write(root.join("STOP-context.json"),serde_json::to_vec_pretty(&serde_json::json!({"seed":seed,"challenger":challenger,"arm":arm,"input":format!("{input:?}"),"pre_events":format!("{events:?}"),"plan":format!("{plan:?}"),"outcome":format!("{outcome:?}")})).unwrap()).unwrap();
           return Err(format!("formal rejection or simulation fault: {outcome:?}"));
          }
         }
         Ok(())
        })();
        let finished=engine.outcome().is_some();let won=matches!(engine.outcome().map(|o|o.winner),Some(Winner::Player(p)) if p==challenger);
        let bytes=engine_native::archive::encode_archive(&engine.save());
        std::fs::write(root.join(format!("final-{seed}-{challenger}-{arm}.ngsa")),&bytes).unwrap();
        let restored=Engine::restore_with_extensions(&engine_native::archive::decode_archive(&bytes).unwrap(),&[&pack]).unwrap();assert_eq!(format!("{:?}",restored.outcome()),format!("{:?}",engine.outcome()));
        let row=serde_json::json!({"seed":seed,"challenger":challenger,"arm":arm,"steps":steps,"finished":finished,"winner":engine.outcome().map(|o|format!("{:?}",o.winner)),"challenger_win":won,"counts":counts,"own_calls":own,"reference_calls":refcalls,"candidate_calls":proposalcalls,"sim_faults":simfaults,"byte_restore":true,"error":game.as_ref().err()});
        writeln!(games,"{row}").unwrap();trace.flush().unwrap();games.flush().unwrap();
        if let Err(error)=game {std::fs::write(root.join("STOP.json"),serde_json::to_vec_pretty(&row).unwrap()).unwrap();panic!("{error}; pilot stops, no retry");}
       }
      }
    }
}

/// Reproducible strategy comparison; run explicitly with --ignored --nocapture --test-threads=1.
/// Latency is diagnostic unless the host is otherwise idle. Percentiles use nearest rank.
#[test]
#[ignore = "manual fixed-seed quality and latency matrix"]
fn nc10_quality_matrix() {
    quality_matrix(8);
}

#[test]
#[ignore = "manual production-budget quality and latency matrix"]
fn nc10_quality_matrix_production() {
    quality_matrix(64);
}

fn quality_matrix(node_budget: usize) {
    use engine_native::{CommandOutcome, Role, Winner};
    use std::time::Instant;
    fn percentile(values: &mut [u128], percentile: usize) -> u128 {
        values.sort_unstable();
        values[(values.len() * percentile).div_ceil(100).saturating_sub(1)]
    }
    let mut matrix_errors = 0;
    for seats in [4, 8] {
        for identity in [false, true] {
            for skills in [false, true] {
                for (variant, depth) in [(0, 1), (1, 1), (1, 2)] {
                    let mut latencies = Vec::new();
                    let (mut games, mut finished, mut wins, mut rejected, mut faults) =
                        (0, 0, 0, 0, 0);
                    let (mut nodes, mut exhausted, mut leaves, mut steps) = (0, 0, 0, 0);
                    for seed in [45000, 45001] {
                        for challenger in 0..seats {
                            let mut e = if identity {
                                Engine::new_identity(seed, seats)
                            } else {
                                Engine::new(seed, seats)
                            };
                            if skills {
                                for seat in 0..seats {
                                    e.assign_general_named(
                                        seat as u8,
                                        ["曹操", "刘备", "孙权", "赵云"][seat % 4],
                                    )
                                    .unwrap();
                                }
                            }
                            let role = e.roles().map(|r| r[challenger]);
                            let probe_before = engine_native::engine::legal_actions::probe_faults();
                            let baseline = BoundedPlanner::new(node_budget).with_policy_variant(0);
                            let candidate = BoundedPlanner::new(node_budget)
                                .with_policy_variant(variant)
                                .with_lookahead(depth);
                            let mut game_steps = 0;
                            while e.outcome().is_none() && game_steps < 800 {
                                let actor = e
                                    .open_choice()
                                    .map(|c| c.observer)
                                    .unwrap_or(e.turn().current);
                                let planner = if actor as usize == challenger {
                                    &candidate
                                } else {
                                    &baseline
                                };
                                let started = Instant::now();
                                let plan = e.plan_action(actor, planner);
                                latencies.push(started.elapsed().as_nanos());
                                faults += plan.sim_faults as u64;
                                if plan.sim_faults > 0 {
                                    eprintln!("QUALITY_FAULT seed={seed} challenger={challenger} step={game_steps} context={:?}", engine_native::ai::sim_fault_log());
                                    engine_native::ai::clear_sim_fault_log();
                                }
                                nodes += plan.nodes_used;
                                exhausted += usize::from(plan.budget_exhausted);
                                leaves += usize::from(plan.nodes_used == 0);
                                let result = e.apply_plan(&plan).unwrap();
                                rejected +=
                                    usize::from(matches!(result, CommandOutcome::Rejected(_)));
                                game_steps += 1;
                                if matches!(result, CommandOutcome::Rejected(_)) {
                                    eprintln!("QUALITY_REJECT seats={seats} seed={seed} actor={actor} plan={plan:?} choice={:?} result={result:?}", e.open_choice());
                                    break;
                                }
                            }
                            faults +=
                                engine_native::engine::legal_actions::probe_faults() - probe_before;
                            games += 1;
                            steps += game_steps;
                            if let Some(outcome) = e.outcome() {
                                finished += 1;
                                wins += usize::from(match outcome.winner {
                                    Winner::Player(seat) => seat as usize == challenger,
                                    Winner::LordSide => {
                                        matches!(role, Some(Role::Lord | Role::Loyalist))
                                    }
                                    Winner::Rebels => role == Some(Role::Rebel),
                                    Winner::Traitor => role == Some(Role::Traitor),
                                    _ => false,
                                });
                            }
                            eprintln!("QUALITY_GAME budget={node_budget} seats={seats} identity={identity} skills={skills} variant={variant} depth={depth} seed={seed} challenger={challenger} role={role:?} steps={game_steps} winner={:?}", e.outcome());
                        }
                    }
                    let p50 = percentile(&mut latencies, 50);
                    let p95 = percentile(&mut latencies, 95);
                    let p99 = percentile(&mut latencies, 99);
                    eprintln!("QUALITY_SUMMARY budget={node_budget} seats={seats} identity={identity} skills={skills} variant={variant} depth={depth} games={games} finished={finished} challenger_wins={wins} steps={steps} samples={} rejected={rejected} faults={faults} nodes={nodes} exhausted={exhausted} zero_node={leaves} p50_ns={p50} p95_ns={p95} p99_ns={p99}", latencies.len());
                    matrix_errors += rejected + faults as usize;
                }
            }
        }
    }
    assert_eq!(
        matrix_errors, 0,
        "quality matrix rejected plans or simulation faults"
    );
}
