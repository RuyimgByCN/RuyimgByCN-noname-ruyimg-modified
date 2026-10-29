//! NC12 基准的各场景（probe/build/drive/copy/archive/ai/long）。

use super::*;

// ---------------------------------------------------------------------------
// 场景：probe —— 三档实际装载能力与目录规模
// ---------------------------------------------------------------------------

pub(super) fn scenario_probe(args: &Args, csv: &mut CsvSink) {
    let wanted: Vec<String> = if args.csv("tiers").is_empty() {
        TIER_TABLE.iter().map(|(n, _)| n.to_string()).collect()
    } else {
        args.csv("tiers")
    };
    println!("# NC12 probe：逐档 import_extensions → bridge_pack → 建局 → 单次规划");
    for tier in &wanted {
        let t_load = Instant::now();
        let host = match load_tier(tier) {
            Ok(h) => h,
            Err(e) => {
                emit_kv("probe", tier, "", &[("result", "FAIL".into()), ("error", e)]);
                continue;
            }
        };
        let load_ns = t_load.elapsed().as_nanos();
        let mut kv: Vec<(&'static str, String)> = vec![
            ("result", "OK".into()),
            ("dirs", tier_dirs_csv(tier)),
            ("load_ns", load_ns.to_string()),
        ];
        if let Some(h) = &host {
            kv.push(("import_ns", h.import_ns.to_string()));
            kv.push(("bridge_ns", h.bridge_ns.to_string()));
            kv.push(("installed_packages", h.installed_total.to_string()));
            kv.push(("skipped_dirs", h.skipped_total.to_string()));
            kv.push(("package_ids", h.package_ids.join("+")));
            kv.push(("content_lock_id", h.lock_id().to_string()));
        }
        let seed = args.u64("seed", 42007);
        let seats = args.usize("seats", 4);
        let t_build = Instant::now();
        match build_engine(tier, host.as_ref(), seed, seats) {
            Ok(mut e) => {
                kv.push(("build_ns", t_build.elapsed().as_nanos().to_string()));
                for (k, v) in catalog_shape(tier, &e) {
                    kv.push((k, v));
                }
                kv.push(("state0", state_shape(&e)));
                let snap = e.save();
                let bytes = encode_archive(&snap);
                kv.push(("archive_bytes", bytes.len().to_string()));
                let planner = planner_of(args.usize("budget", 64), args.u8("lookahead", 1), args.u8("variant", 0));
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let t_plan = Instant::now();
                let plan = e.plan_action(seat, &planner);
                kv.push(("plan_smoke_ns", t_plan.elapsed().as_nanos().to_string()));
                kv.push(("plan_smoke_nodes", plan.nodes_used.to_string()));
                let out = e.apply_plan(&plan);
                kv.push(("apply_smoke", format!("{out:?}").split(['(', ' ']).next().unwrap_or("?").to_string()));
                // 建局计时（为 probe 提供最小样本，正式样本见 build 场景）。
                let samples = args.usize("samples", 5);
                let mut build_samples = Vec::new();
                for i in 0..samples {
                    let t = Instant::now();
                    let r = build_engine(tier, host.as_ref(), seed + i as u64, seats);
                    build_samples.push(t.elapsed().as_nanos());
                    std::hint::black_box(r.is_ok());
                }
                csv.push(&format!("probe_build_ns_{tier}"), &build_samples);
                emit_stats("probe", tier, "", "build_ns", &stats(&build_samples));
                emit_kv("probe", tier, "", &kv);
            }
            Err(e) => {
                kv.push(("build", "FAIL".into()));
                kv.push(("error", e));
                emit_kv("probe", tier, "", &kv);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 场景：build —— 启动/建局（无扩展 / 单扩展 / 常用组合）
// ---------------------------------------------------------------------------

pub(super) fn scenario_build(args: &Args, csv: &mut CsvSink) {
    let tiers: Vec<String> = if args.csv("tiers").is_empty() {
        TIER_TABLE.iter().map(|(n, _)| n.to_string()).collect()
    } else {
        args.csv("tiers")
    };
    let samples = args.usize("samples", 30);
    let seats = args.usize("seats", 4);
    let seed = args.u64("seed", 42007);
    println!("# NC12 build：标准目录构建 + 三档建局（samples={samples}, seats={seats}, seed={seed}）");
    // 标准目录单独计时（`Engine::new` 内部同样构建一次）。
    let mut cat = Vec::new();
    for _ in 0..samples {
        let t = Instant::now();
        let c = standard_catalog();
        cat.push(t.elapsed().as_nanos());
        std::hint::black_box(c.ruleset_version);
    }
    csv.push("standard_catalog_ns", &cat);
    emit_stats("build", "none", &config_str(seats, 0, 0, 0), "standard_catalog_ns", &stats(&cat));

    for tier in &tiers {
        let t_load = Instant::now();
        let host = match load_tier(tier) {
            Ok(h) => h,
            Err(e) => {
                emit_kv("build", tier, "", &[("result", "FAIL".into()), ("error", e)]);
                continue;
            }
        };
        let load_ns = t_load.elapsed().as_nanos();
        let mut samples_ns = Vec::new();
        let mut first_err: Option<String> = None;
        let mut ruleset = 0u64;
        let mut bytes = 0usize;
        let mut shape = String::new();
        let mut cat_kv: Vec<(&'static str, String)> = Vec::new();
        for i in 0..samples {
            let t = Instant::now();
            let r = build_engine(tier, host.as_ref(), seed + i as u64, seats);
            samples_ns.push(t.elapsed().as_nanos());
            match r {
                Ok(e) => {
                    ruleset = e.ruleset_version();
                    if i == 0 {
                        bytes = encode_archive(&e.save()).len();
                        shape = state_shape(&e);
                        cat_kv = catalog_shape(tier, &e);
                    }
                }
                Err(err) => {
                    if first_err.is_none() {
                        first_err = Some(err);
                    }
                }
            }
        }
        let mut kv: Vec<(&'static str, String)> = vec![
            ("dirs", tier_dirs_csv(tier)),
            ("seats", seats.to_string()),
            ("samples", samples.to_string()),
            ("host_load_ns", load_ns.to_string()),
            ("ruleset_version", ruleset.to_string()),
            ("archive_bytes_at_open", bytes.to_string()),
            ("state_at_open", shape),
            ("first_error", first_err.unwrap_or_else(|| "-".into())),
        ];
        if let Some(h) = &host {
            kv.push(("import_ns", h.import_ns.to_string()));
            kv.push(("bridge_ns", h.bridge_ns.to_string()));
            kv.push(("package_ids", h.package_ids.join("+")));
            kv.push(("installed_packages", h.installed_total.to_string()));
            kv.push(("content_lock_id", h.lock_id().to_string()));
        }
        kv.extend(cat_kv);
        csv.push(&format!("build_ns_{tier}"), &samples_ns);
        emit_stats("build", tier, &config_str(seats, 0, 0, 0), "engine_build_ns", &stats(&samples_ns));
        emit_kv("build", tier, &config_str(seats, 0, 0, 0), &kv);
    }
}

// ---------------------------------------------------------------------------
// 场景：drive —— 整局规划驱动，每命令尾延迟
// ---------------------------------------------------------------------------

pub(super) fn scenario_drive(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let budget = args.usize("budget", 64);
    let lookahead = args.u8("lookahead", 1);
    let variant = args.u8("variant", 0);
    let games = args.usize("games", 1);
    let seed = args.u64("seed", 42007);
    let max_steps = args.usize("max_steps", 4000);
    let wall_secs = args.u64("wall_secs", 600);
    let cfg = config_str(seats, budget, lookahead, variant);
    let trace = args.get("trace").is_some();
    let catch_panics = args.get("catch_panics").is_some();
    println!(
        "# NC12 drive：tier={tier} dirs={} {cfg} games={games} seed={seed} max_steps={max_steps} wall_secs={wall_secs}",
        tier_dirs_csv(&tier)
    );
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("drive", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let planner = planner_of(budget, lookahead, variant);
    let mut games_out: Vec<GameRun> = Vec::new();
    for g in 0..games {
        let mut e = match build_engine(&tier, host.as_ref(), seed + g as u64, seats) {
            Ok(e) => e,
            Err(err) => {
                emit_kv("drive", &tier, &format!("{cfg},game={g}"), &[("result", "FAIL".into()), ("error", err)]);
                continue;
            }
        };
        let run = drive_game(
            &mut e,
            &planner,
            max_steps,
            wall_secs as u128 * 1_000_000_000,
            &mut |_e, _step| {},
            0,
            trace,
            catch_panics,
        );
        println!(
            "RESULT\tscenario=drive\ttier={tier}\tconfig={cfg},game={g}\tgame={g}\tsteps={}\taccepted={}\tpending={}\trejected={}\tfinished={}\tstalled={}\tplan_panics={}\tdegraded={}\twall_ns={}\tevents={}\tnodes_total={}\tfinal_state={}",
            run.steps, run.accepted, run.pending, run.rejected, run.finished, run.stalled,
            run.panics.len(), !run.panics.is_empty(),
            run.wall_ns, run.events, run.total_nodes(), run.final_shape
        );
        if !run.plan_ns.is_empty() {
            csv.push(&format!("drive_{tier}_plan_ns_g{g}"), &run.plan_ns);
            csv.push(&format!("drive_{tier}_submit_ns_g{g}"), &run.submit_ns);
            csv.push(&format!("drive_{tier}_cmd_ns_g{g}"), &run.cmd_ns);
            emit_stats("drive", &tier, &format!("{cfg},game={g}"), "plan_ns", &stats(&run.plan_ns));
            emit_stats("drive", &tier, &format!("{cfg},game={g}"), "submit_ns", &stats(&run.submit_ns));
            emit_stats("drive", &tier, &format!("{cfg},game={g}"), "cmd_ns", &stats(&run.cmd_ns));
        }
        emit_kv("drive", &tier, &format!("{cfg},game={g}"), &run.summary_kv());
        games_out.push(run);
    }
    // 跨局汇总（样本拼接；不得与其它档位混算）。
    let mut plan_all = Vec::new();
    let mut submit_all = Vec::new();
    let mut cmd_all = Vec::new();
    for r in &games_out {
        plan_all.extend_from_slice(&r.plan_ns);
        submit_all.extend_from_slice(&r.submit_ns);
        cmd_all.extend_from_slice(&r.cmd_ns);
    }
    if !cmd_all.is_empty() {
        emit_stats("drive", &tier, &format!("{cfg},all_games"), "plan_ns", &stats(&plan_all));
        emit_stats("drive", &tier, &format!("{cfg},all_games"), "submit_ns", &stats(&submit_all));
        emit_stats("drive", &tier, &format!("{cfg},all_games"), "cmd_ns", &stats(&cmd_all));
        let total_wall: u128 = games_out.iter().map(|r| r.wall_ns).sum();
        let total_steps: usize = games_out.iter().map(|r| r.steps).sum();
        let total_nodes: u128 = games_out.iter().map(|r| r.total_nodes()).sum();
        emit_kv(
            "drive",
            &tier,
            &format!("{cfg},all_games"),
            &[
                ("result", "OK".into()),
                ("games", games_out.len().to_string()),
                ("steps", total_steps.to_string()),
                ("finished_games", games_out.iter().filter(|r| r.finished).count().to_string()),
                ("stalled_games", games_out.iter().filter(|r| r.stalled).count().to_string()),
                ("plan_panics", games_out.iter().map(|r| r.panics.len()).sum::<usize>().to_string()),
                ("degraded", games_out.iter().any(|r| !r.panics.is_empty()).to_string()),
                ("wall_ns", total_wall.to_string()),
                ("ns_per_step_wall", if total_steps == 0 { 0 } else { total_wall / total_steps as u128 }.to_string()),
                ("nodes_total", total_nodes.to_string()),
                ("nodes_per_sec_plan", {
                    let plan_sum: u128 = plan_all.iter().sum();
                    if plan_sum == 0 { 0 } else { total_nodes * 1_000_000_000 / plan_sum }
                }
                .to_string()),
                ("events", games_out.iter().map(|r| r.events).sum::<usize>().to_string()),
            ],
        );
    }
}

// ---------------------------------------------------------------------------
// 场景：copy —— 状态复制 / 模拟 / 状态规模增长
// ---------------------------------------------------------------------------

pub(super) fn scenario_copy(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let budget = args.usize("budget", 64);
    let lookahead = args.u8("lookahead", 1);
    let variant = args.u8("variant", 0);
    let steps = args.usize("steps", 60);
    let samples = args.usize("samples", 30);
    let seed = args.u64("seed", 42007);
    let max_steps = args.usize("max_steps", 4000);
    let wall_secs = args.u64("wall_secs", 600);
    let cfg = config_str(seats, budget, lookahead, variant);
    let trace = args.get("trace").is_some();
    let catch_panics = args.get("catch_panics").is_some();
    println!("# NC12 copy：tier={tier} dirs={} {cfg} steps={steps} samples={samples}", tier_dirs_csv(&tier));
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("copy", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let mut e = match build_engine(&tier, host.as_ref(), seed, seats) {
        Ok(e) => e,
        Err(err) => {
            emit_kv("copy", &tier, &cfg, &[("result", "FAIL".into()), ("error", err)]);
            return;
        }
    };
    let planner = planner_of(budget, lookahead, variant);

    // 开局（步数=0）与推进 steps 步后的对照。
    let stages: Vec<(&str, usize)> = vec![("open", 0), ("mid", steps)];
    let mut done_steps = 0usize;
    for (stage, target) in stages {
        if target > done_steps {
            while done_steps < target && e.outcome().is_none() {
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let plan = e.plan_action(seat, &planner);
                if e.apply_plan(&plan).is_err() {
                    break;
                }
                done_steps += 1;
            }
        }
        let stage_cfg = format!("{cfg},stage={stage},steps={done_steps}");
        emit_kv(
            "copy",
            &tier,
            &stage_cfg,
            &[
                ("state", state_shape(&e)),
                ("finished", e.outcome().is_some().to_string()),
            ],
        );
        // save()：快照克隆
        let mut save_ns = Vec::new();
        let mut acc = 0usize;
        for _ in 0..samples {
            let t = Instant::now();
            let s = e.save();
            save_ns.push(t.elapsed().as_nanos());
            acc += s.core.players.len();
        }
        // Engine::clone()：规划隔离的实际复制面
        let mut clone_ns = Vec::new();
        for _ in 0..samples {
            let t = Instant::now();
            let c = e.clone();
            clone_ns.push(t.elapsed().as_nanos());
            acc += c.revision() as usize;
        }
        // plan_action：克隆 + 未知牌重分配 + 候选模拟
        let mut plan_ns = Vec::new();
        let mut nodes = Vec::new();
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        for _ in 0..samples {
            let t = Instant::now();
            let p = e.plan_action(seat, &planner);
            plan_ns.push(t.elapsed().as_nanos());
            nodes.push(p.nodes_used as u128);
            std::hint::black_box(p.action.clone());
        }
        // 归档编码（字节存档）
        let mut enc_ns = Vec::new();
        for _ in 0..samples {
            let t = Instant::now();
            let s = e.save();
            let bytes = encode_archive(&s);
            enc_ns.push(t.elapsed().as_nanos());
            acc += bytes.len();
        }
        let bytes = encode_archive(&e.save()).len();
        csv.push(&format!("copy_{tier}_{stage}_save_ns"), &save_ns);
        csv.push(&format!("copy_{tier}_{stage}_clone_ns"), &clone_ns);
        csv.push(&format!("copy_{tier}_{stage}_plan_ns"), &plan_ns);
        csv.push(&format!("copy_{tier}_{stage}_encode_ns"), &enc_ns);
        emit_stats("copy", &tier, &stage_cfg, "save_ns", &stats(&save_ns));
        emit_stats("copy", &tier, &stage_cfg, "engine_clone_ns", &stats(&clone_ns));
        emit_stats("copy", &tier, &stage_cfg, "plan_action_ns", &stats(&plan_ns));
        emit_stats("copy", &tier, &stage_cfg, "encode_archive_ns", &stats(&enc_ns));
        emit_kv(
            "copy",
            &tier,
            &stage_cfg,
            &[
                ("archive_bytes", bytes.to_string()),
                ("plan_nodes_p50", stats(&nodes).p50.to_string()),
                ("plan_nodes_max", stats(&nodes).max.to_string()),
                ("nodes_per_sec_plan", {
                    let plan_sum: u128 = plan_ns.iter().sum();
                    if plan_sum == 0 { 0 } else { nodes.iter().sum::<u128>() * 1_000_000_000 / plan_sum }
                }
                .to_string()),
                ("black_box_acc", acc.to_string()),
            ],
        );
    }

    // 推进过程中状态规模增长曲线（players/frames/skills 等）。
    let mut e2 = match build_engine(&tier, host.as_ref(), seed, seats) {
        Ok(e) => e,
        Err(err) => {
            emit_kv("copy", &tier, &format!("{cfg},growth"), &[("result", "FAIL".into()), ("error", err)]);
            return;
        }
    };
    let every = args.usize("growth_every", 20);
    let mut grass = Vec::new();
    let mut r = drive_game(
        &mut e2,
        &planner,
        max_steps,
        wall_secs as u128 * 1_000_000_000,
        &mut |eng: &Engine, step: usize| {
            grass.push((step, state_shape(eng), encode_archive(&eng.save()).len()));
        },
        every,
        trace,
        catch_panics,
    );
    r.final_shape = state_shape(&e2);
    println!(
        "RESULT\tscenario=copy\ttier={tier}\tconfig={cfg},growth\tsteps={}\tfinished={}\tstalled={}\twall_ns={}\tfinal_state={}",
        r.steps, r.finished, r.stalled, r.wall_ns, r.final_shape
    );
    for (step, shape, bytes) in &grass {
        println!("RESULT\tscenario=copy\ttier={tier}\tconfig={cfg},growth\tstep={step}\tarchive_bytes={bytes}\tstate={shape}");
    }
}

/// 所有权校验**计入**的公开牌区（`snap/validation.rs::validate_card_ownership`
/// 口径）：手牌/装备/判定/离场区 + 牌库/弃牌堆/结算牌桌 + 挂起机器持牌。
/// 未计入的特殊区（炼妖壶收纳、筑城、卡牌改造表等）单独标注，避免把
/// 「校验器不认的区」误判成「牌区重复」。
pub(super) fn counted_zones(snap: &engine_native::StateSnapshot, card: u32, name: &str) -> String {
    let count = |v: &Vec<u32>| v.iter().filter(|c| **c == card).count();
    let mut z: Vec<String> = Vec::new();
    let core = &snap.core;
    let push = |label: String, n: usize, out: &mut Vec<String>| {
        if n > 0 {
            out.push(format!("{label}={n}"));
        }
    };
    push("deck".into(), count(&core.deck), &mut z);
    push("discard".into(), count(&core.discard), &mut z);
    push("table".into(), count(&core.table), &mut z);
    let mut uncounted: Vec<String> = Vec::new();
    for (i, p) in core.players.iter().enumerate() {
        push(format!("hand[{i}]"), count(&p.hand), &mut z);
        push(format!("equips[{i}]"), count(&p.equips), &mut z);
        push(format!("judge[{i}]"), count(&p.judge), &mut z);
        push(format!("off_court[{i}]"), count(&p.off_court), &mut z);
        push(format!("lianyaohu_store[{i}]"), count(&p.lianyaohu_store), &mut uncounted);
        push(format!("zhucheng[{i}]"), count(&p.zhucheng), &mut uncounted);
    }
    if core.declared_kinds.contains_key(&card) {
        uncounted.push("declared_kinds".into());
    }
    if core.declared_slots.contains_key(&card) {
        uncounted.push("declared_slots".into());
    }
    if core.card_natures.contains_key(&card) {
        uncounted.push("card_natures".into());
    }
    format!(
        "counted=[{}] uncounted=[{}] parks={} card_name={}",
        z.join("+"),
        uncounted.join("+"),
        snap.resolution.parks.len(),
        name
    )
}

// ---------------------------------------------------------------------------
// 场景：archive —— 存读档（含扩展目录）与归档字节
// ---------------------------------------------------------------------------

pub(super) fn scenario_archive(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let samples = args.usize("samples", 30);
    let seed = args.u64("seed", 42007);
    let steps = args.usize("steps", 80);
    let budget = args.usize("budget", 64);
    let cfg = format!("seats={seats},samples={samples},steps={steps}");
    println!("# NC12 archive：tier={tier} dirs={} {cfg}", tier_dirs_csv(&tier));
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("archive", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let mut e = match build_engine(&tier, host.as_ref(), seed, seats) {
        Ok(e) => e,
        Err(err) => {
            emit_kv("archive", &tier, &cfg, &[("result", "FAIL".into()), ("error", err)]);
            return;
        }
    };
    let planner = planner_of(budget, 1, 0);
    let mut stage_steps = 0usize;
    for stage in ["open", "played"] {
        if stage == "played" {
            while stage_steps < steps && e.outcome().is_none() {
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let plan = e.plan_action(seat, &planner);
                if e.apply_plan(&plan).is_err() {
                    break;
                }
                stage_steps += 1;
            }
        }
        let stage_cfg = format!("{cfg},stage={stage},steps={stage_steps}");
        let mut save_ns = Vec::new();
        let mut enc_ns = Vec::new();
        let mut dec_ns = Vec::new();
        let mut restore_ns = Vec::new();
        let mut roundtrip_fail = 0usize;
        let mut hash_mismatch = 0usize;
        let mut acc = 0usize;
        for _ in 0..samples {
            let t = Instant::now();
            let snap = e.save();
            save_ns.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let bytes = encode_archive(&snap);
            enc_ns.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let decoded = match decode_archive(&bytes) {
                Ok(d) => d,
                Err(err) => {
                    roundtrip_fail += 1;
                    eprintln!("decode_archive 失败: {err:?}");
                    continue;
                }
            };
            dec_ns.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let restored = match host.as_ref() {
                None => Engine::restore(&decoded),
                Some(h) => {
                    let refs = h.refs();
                    Engine::restore_with_extensions(&decoded, &refs)
                }
            };
            restore_ns.push(t.elapsed().as_nanos());
            match restored {
                Ok(r) => {
                    if r.state_hash() != e.state_hash() {
                        hash_mismatch += 1;
                    }
                    acc += r.revision() as usize;
                }
                Err(err) => {
                    roundtrip_fail += 1;
                    eprintln!("restore 失败: {err:?}");
                    if let engine_native::SnapshotError::IdOutOfBounds { what, got } = &err {
                        let name = e
                            .catalog()
                            .cards
                            .get(*got as usize)
                            .map(|c| c.name)
                            .unwrap_or("<越界牌 id>");
                        println!(
                            "RESULT\tscenario=archive\ttier={tier}\tconfig={stage_cfg}\tdiagnostic=restore_reject\twhat={what}\tcard={got}\tcard_name={name}\t{}",
                            counted_zones(&decoded, *got as u32, name)
                        );
                    }
                }
            }
        }
        let bytes = encode_archive(&e.save()).len();
        csv.push(&format!("archive_{tier}_{stage}_save_ns"), &save_ns);
        csv.push(&format!("archive_{tier}_{stage}_encode_ns"), &enc_ns);
        csv.push(&format!("archive_{tier}_{stage}_decode_ns"), &dec_ns);
        csv.push(&format!("archive_{tier}_{stage}_restore_ns"), &restore_ns);
        emit_stats("archive", &tier, &stage_cfg, "save_ns", &stats(&save_ns));
        emit_stats("archive", &tier, &stage_cfg, "encode_archive_ns", &stats(&enc_ns));
        emit_stats("archive", &tier, &stage_cfg, "decode_archive_ns", &stats(&dec_ns));
        emit_stats("archive", &tier, &stage_cfg, "restore_ns", &stats(&restore_ns));
        emit_kv(
            "archive",
            &tier,
            &stage_cfg,
            &[
                ("result", if roundtrip_fail == 0 && hash_mismatch == 0 { "OK".into() } else { "FAIL".into() }),
                ("archive_bytes", bytes.to_string()),
                ("roundtrip_failures", roundtrip_fail.to_string()),
                ("state_hash_mismatch", hash_mismatch.to_string()),
                ("state_hash", hex32(&e.state_hash())),
                ("state", state_shape(&e)),
                ("roundtrip_token", acc.to_string()),
            ],
        );
    }
}

// ---------------------------------------------------------------------------
// 场景：ai —— 同种子同预算的策略矩阵（lookahead 1/2 × variant 0/1）
// ---------------------------------------------------------------------------

pub(super) fn scenario_ai(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let seed = args.u64("seed", 42007);
    let cmds = args.usize("cmds", 200);
    let budgets = if args.csv("budgets").is_empty() {
        vec![args.usize("budget", 64)]
    } else {
        args.csv("budgets").iter().filter_map(|s| s.parse().ok()).collect()
    };
    let configs_arg = args.csv("configs");
    let configs: Vec<(u8, u8)> = if configs_arg.is_empty() {
        vec![(1, 0), (1, 1), (2, 0), (2, 1)]
    } else {
        configs_arg
            .iter()
            .filter_map(|c| c.split_once('/').and_then(|(l, v)| Some((l.parse().ok()?, v.parse().ok()?))))
            .collect()
    };
    println!(
        "# NC12 ai：tier={tier} dirs={} seats={seats} seed={seed} cmds={cmds} budgets={:?} configs={:?}",
        tier_dirs_csv(&tier),
        budgets,
        configs
    );
    let catch_panics = args.get("catch_panics").is_some();
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("ai", &tier, "", &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let mut decision_sequences: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for &budget in &budgets {
        for &(lookahead, variant) in &configs {
            let cfg = config_str(seats, budget, lookahead, variant);
            let mut e = match build_engine(&tier, host.as_ref(), seed, seats) {
                Ok(e) => e,
                Err(err) => {
                    emit_kv("ai", &tier, &cfg, &[("result", "FAIL".into()), ("error", err)]);
                    continue;
                }
            };
            let planner = planner_of(budget, lookahead, variant);
            let mut plan_ns = Vec::new();
            let mut nodes = Vec::new();
            let mut decisions = Vec::new();
            let mut accepted = 0usize;
            let mut rejected = 0usize;
            let mut steps = 0usize;
            let mut panics: Vec<(usize, u8, String)> = Vec::new();
            let t_all = Instant::now();
            while steps < cmds && e.outcome().is_none() {
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let t = Instant::now();
                let (plan, panic_msg) = plan_with_catch(&e, seat, &planner, catch_panics, steps);
                if let Some(msg) = panic_msg {
                    panics.push((steps, seat, msg));
                }
                plan_ns.push(t.elapsed().as_nanos());
                nodes.push(plan.nodes_used as u128);
                decisions.push(format!("{:?}", plan.action));
                match e.apply_plan(&plan) {
                    Ok(CommandOutcome::Rejected(_)) | Err(_) => rejected += 1,
                    Ok(_) => accepted += 1,
                }
                steps += 1;
            }
            let wall_ns = t_all.elapsed().as_nanos();
            let total_nodes: u128 = nodes.iter().sum();
            let plan_sum: u128 = plan_ns.iter().sum();
            csv.push(&format!("ai_{tier}_b{budget}_l{lookahead}_v{variant}_plan_ns"), &plan_ns);
            emit_stats("ai", &tier, &cfg, "plan_action_ns", &stats(&plan_ns));
            let ns = stats(&nodes);
            emit_kv(
                "ai",
                &tier,
                &cfg,
                &[
                    ("metric", "nodes_per_plan".into()),
                    ("n", ns.n.to_string()),
                    ("min", ns.min.to_string()),
                    ("p50", ns.p50.to_string()),
                    ("p95", ns.p95.to_string()),
                    ("max", ns.max.to_string()),
                    ("mean", ns.mean.to_string()),
                    ("total", ns.sum.to_string()),
                ],
            );
            emit_kv(
                "ai",
                &tier,
                &cfg,
                &[
                    ("result", "OK".into()),
                    ("steps", steps.to_string()),
                    ("accepted", accepted.to_string()),
                    ("rejected", rejected.to_string()),
                    ("finished", e.outcome().is_some().to_string()),
                    ("plan_panics", panics.len().to_string()),
                    ("degraded", (!panics.is_empty()).to_string()),
                    ("panic_steps", if panics.is_empty() {
                        "-".to_string()
                    } else {
                        panics.iter().map(|(s, seat, _)| format!("{s}@seat{seat}")).collect::<Vec<_>>().join("|")
                    }),
                    ("wall_ns", wall_ns.to_string()),
                    ("plan_total_ns", plan_sum.to_string()),
                    ("nodes_total", total_nodes.to_string()),
                    (
                        "nodes_per_sec",
                        if plan_sum == 0 { 0 } else { total_nodes * 1_000_000_000 / plan_sum }.to_string(),
                    ),
                    (
                        "ns_per_step_wall",
                        if steps == 0 { 0 } else { wall_ns / steps as u128 }.to_string(),
                    ),
                    ("final_state", state_shape(&e)),
                ],
            );
            decision_sequences.insert(cfg, decisions);
        }
    }
    // 同种子下策略差异：与「lookahead=1,variant=0」逐步对比。
    for &budget in &budgets {
        let base_key = config_str(seats, budget, 1, 0);
        let Some(base) = decision_sequences.get(&base_key) else { continue };
        for &(lookahead, variant) in &configs {
            let cfg = config_str(seats, budget, lookahead, variant);
            let Some(cur) = decision_sequences.get(&cfg) else { continue };
            let common = base.len().min(cur.len());
            let diff = (0..common).filter(|&i| base[i] != cur[i]).count();
            emit_kv(
                "ai",
                &tier,
                &cfg,
                &[
                    ("compare_base", base_key.clone()),
                    ("compare_steps", common.to_string()),
                    ("decision_diff_steps", diff.to_string()),
                    ("decision_len_base", base.len().to_string()),
                    ("decision_len_cur", cur.len().to_string()),
                ],
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 场景：long —— 长局内存（RSS 增长）
// ---------------------------------------------------------------------------

pub(super) fn scenario_long(args: &Args, _csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let budget = args.usize("budget", 8);
    let min_cmds = args.usize("min_cmds", 300);
    let max_games = args.usize("max_games", 4);
    let seed = args.u64("seed", 42007);
    let every = args.usize("sample_every", 25);
    let trace = args.get("trace").is_some();
    let catch_panics = args.get("catch_panics").is_some();
    let cfg = config_str(seats, budget, 1, 0);
    println!(
        "# NC12 long：tier={tier} dirs={} {cfg} min_cmds={min_cmds} max_games={max_games} seed={seed} sample_every={every}",
        tier_dirs_csv(&tier)
    );
    let rss0 = rss_kb();
    let heap0 = heap_snapshot();
    emit_kv(
        "long",
        &tier,
        &cfg,
        &[
            ("phase", "process_start".into()),
            ("rss_kb", rss0.map(|v| v.to_string()).unwrap_or_else(|| "NA".into())),
            ("rss_source", "ps -o rss= (沙箱禁止时 NA；外部用 /usr/bin/time -l)".into()),
            ("alloc_counting", alloc_counting_on().to_string()),
            ("heap_allocated_bytes", heap0.0.to_string()),
            ("heap_freed_bytes", heap0.1.to_string()),
            ("heap_alloc_calls", heap0.2.to_string()),
            ("pid", std::process::id().to_string()),
        ],
    );
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("long", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let planner = planner_of(budget, 1, 0);
    let mut total_cmds = 0usize;
    let mut rss_peak = rss0.unwrap_or(0);
    let mut rss_series: Vec<(usize, u64, String)> = Vec::new();
    for g in 0..max_games {
        if total_cmds >= min_cmds {
            break;
        }
        let mut e = match build_engine(&tier, host.as_ref(), seed + g as u64, seats) {
            Ok(e) => e,
            Err(err) => {
                emit_kv("long", &tier, &format!("{cfg},game={g}"), &[("result", "FAIL".into()), ("error", err)]);
                return;
            }
        };
        let rss_before = rss_kb().unwrap_or(0);
        let mut samples: Vec<(usize, u64, String)> = Vec::new();
        let mut run = drive_game(
            &mut e,
            &planner,
            args.usize("max_steps", 4000),
            args.u64("wall_secs", 600) as u128 * 1_000_000_000,
            &mut |eng: &Engine, step: usize| {
                if let Some(rss) = rss_kb() {
                    samples.push((step, rss, state_shape(eng)));
                }
            },
            every,
            trace,
            catch_panics,
        );
        run.final_shape = state_shape(&e);
        let rss_after = rss_kb().unwrap_or(0);
        rss_peak = rss_peak.max(rss_after);
        total_cmds += run.steps;
        let heap_game = heap_snapshot();
        println!(
            "RESULT\tscenario=long\ttier={tier}\tconfig={cfg},game={g}\tgame={g}\tsteps={}\taccepted={}\tfinished={}\tstalled={}\tplan_panics={}\tsim_faults={}\twall_ns={}\trss_before_kb={rss_before}\trss_after_kb={rss_after}\trss_delta_kb={}\theap_allocated_bytes={}\theap_freed_bytes={}\theap_live_bytes={}\tfinal_state={}",
            run.steps, run.accepted, run.finished, run.stalled, run.panics.len(), run.sim_faults, run.wall_ns,
            rss_after as i64 - rss_before as i64,
            heap_game.0, heap_game.1, heap_game.0.saturating_sub(heap_game.1),
            run.final_shape
        );
        for (step, rss, shape) in &samples {
            let heap = heap_snapshot();
            println!(
                "RESULT\tscenario=long\ttier={tier}\tconfig={cfg},game={g}\tstep={step}\trss_kb={rss}\theap_live_bytes={}\tstate={shape}",
                heap.0.saturating_sub(heap.1)
            );
            rss_series.push((*step, *rss, shape.clone()));
        }
        emit_stats(
            "long",
            &tier,
            &format!("{cfg},game={g}"),
            "plan_action_ns",
            &stats(&run.plan_ns),
        );
    }
    // 同场景重复后的资源累积检查：额外跑 20 次 存档+编码 往返。
    let mut acc = 0usize;
    if let Ok(e) = build_engine(&tier, host.as_ref(), seed, seats) {
        let t = Instant::now();
        for _ in 0..20 {
            let snap = e.save();
            let bytes = encode_archive(&snap);
            acc += bytes.len() + snap.core.players.len();
        }
        emit_kv(
            "long",
            &tier,
            &cfg,
            &[
                ("phase", "post_save_cycles".into()),
                ("save_cycle_total_ns", t.elapsed().as_nanos().to_string()),
                ("rss_kb", rss_kb().map(|v| v.to_string()).unwrap_or_else(|| "NA".into())),
                ("token", acc.to_string()),
            ],
        );
    }
    let rss_end = rss_kb().unwrap_or(0);
    rss_peak = rss_peak.max(rss_end);
    let heap1 = heap_snapshot();
    emit_kv(
        "long",
        &tier,
        &cfg,
        &[
            ("result", "OK".into()),
            ("total_cmds", total_cmds.to_string()),
            ("alloc_counting", alloc_counting_on().to_string()),
            ("heap_allocated_bytes", heap1.0.saturating_sub(heap0.0).to_string()),
            ("heap_freed_bytes", heap1.1.saturating_sub(heap0.1).to_string()),
            ("heap_live_delta_bytes", heap1.0.saturating_sub(heap0.0).saturating_sub(heap1.1.saturating_sub(heap0.1)).to_string()),
            ("heap_alloc_calls", heap1.2.to_string()),
            ("heap_peak_live_bytes", heap1.3.to_string()),
            ("rss_start_kb", rss0.map(|v| v.to_string()).unwrap_or_else(|| "NA".into())),
            ("rss_end_kb", rss_end.to_string()),
            ("rss_peak_kb", rss_peak.to_string()),
            (
                "rss_delta_kb",
                rss0.map(|v| rss_end as i64 - v as i64).unwrap_or(0).to_string(),
            ),
            ("rss_samples", rss_series.len().to_string()),
            ("min_cmds_reached", (total_cmds >= min_cmds).to_string()),
        ],
    );
}
