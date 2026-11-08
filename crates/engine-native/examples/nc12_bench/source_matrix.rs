//! NC12 基准场景：`source_matrix` —— 37 来源 / 46 扩展包同机同负载前后性能对比（`none` 基线 vs 逐来源/逐包启用）。

use super::*;

pub(super) fn scenario_source_matrix(args: &Args, csv: &mut CsvSink) {
    let seats = args.usize("seats", 4);
    let seed = args.u64("seed", 42007);
    let build_samples = args.usize("build_samples", 20);
    let drive_steps = args.usize("drive_steps", 80);
    let archive_samples = args.usize("archive_samples", 20);
    let budget = args.usize("budget", 64);
    let cfg = config_str(seats, budget, 1, 0);

    let root = repo_extensions_root();
    let map_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/operations/evidence/2026-10-19-package-source-map/map.tsv");
    let map_text = std::fs::read_to_string(&map_path).expect("read map.tsv");

    let tmp = tempfile::tempdir().expect("tempdir");
    let store = content_runtime::PackageStore::open(tmp.path().join("store")).expect("open store");
    let t_import = Instant::now();
    let report = content_runtime::import_extensions(&store, &root, &install_options())
        .expect("import_extensions");
    let import_all_ns = t_import.elapsed().as_nanos();

    let mut by_dir: BTreeMap<String, PathBuf> = BTreeMap::new();
    for meta in &report.installed {
        if let Some(ext_root) = &meta.extension_root {
            let p = PathBuf::from(ext_root);
            if let Some(name) = p.file_name().map(|s| s.to_string_lossy().into_owned()) {
                by_dir.insert(name, p);
            }
        }
    }

    // Build target list: ("none", vec![]) + 37 sources ("src:L01", dirs) + multi-pack individual dirs ("pkg:<dir>", vec![dir])
    let mut targets: Vec<(String, String, Vec<String>)> = vec![(
        "none".to_string(),
        "standard-baseline".to_string(),
        Vec::new(),
    )];
    let mut multi_subpkgs: Vec<(String, String, Vec<String>)> = Vec::new();
    for (idx, line) in map_text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let src_id = cols[0].trim().to_string();
        let src_name = cols[1].trim().to_string();
        let pkgs: Vec<String> = cols[3]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if pkgs.len() > 1 {
            for p in &pkgs {
                multi_subpkgs.push((
                    format!("pkg:{p}"),
                    format!("{src_id}:{p}"),
                    vec![p.clone()],
                ));
            }
        }
        targets.push((format!("src:{src_id}"), src_name, pkgs));
    }
    targets.extend(multi_subpkgs);

    println!(
        "# NC12 source_matrix：import_all_ns={import_all_ns} installed={} targets={} (none + 37 sources + 11 multi-source subpackages)",
        report.installed.len(),
        targets.len()
    );

    let planner = planner_of(budget, 1, 0);
    for (tier_id, label, dirs) in &targets {
        let t_bridge = Instant::now();
        let mut packs = Vec::new();
        for d in dirs {
            let path = by_dir
                .get(d)
                .unwrap_or_else(|| panic!("missing installed dir {d}"));
            let pack = load_pack_dir(path).unwrap_or_else(|e| panic!("{d}: {e}"));
            packs.push(pack);
        }
        let bridge_ns = if dirs.is_empty() {
            0
        } else {
            t_bridge.elapsed().as_nanos()
        };
        let refs: Vec<&ExtensionPack> = packs.iter().collect();

        // 1. Build benchmark
        let mut build_ns = Vec::with_capacity(build_samples);
        let mut cards_len = 0usize;
        let mut skills_len = 0usize;
        let mut generals_len = 0usize;
        let mut ruleset = 0u64;
        for i in 0..build_samples {
            let t = Instant::now();
            let e = if refs.is_empty() {
                Engine::new(seed + i as u64, seats)
            } else {
                Engine::new_with_extensions(seed + i as u64, seats, &refs)
                    .unwrap_or_else(|err| panic!("{tier_id}: build failed: {err:?}"))
            };
            build_ns.push(t.elapsed().as_nanos());
            if i == 0 {
                cards_len = e.catalog().cards.len();
                skills_len = e.catalog().skills.len();
                generals_len = e.catalog().extension_generals.len();
                ruleset = e.ruleset_version();
            }
        }

        // 2. Drive benchmark
        let mut e = if refs.is_empty() {
            Engine::new(seed, seats)
        } else {
            Engine::new_with_extensions(seed, seats, &refs).expect("build for drive")
        };
        let run = drive_game(
            &mut e,
            &planner,
            drive_steps,
            30_000_000_000,
            &mut |_, _| {},
            0,
            false,
            true,
        );

        // 3. Archive save/encode/decode/restore benchmark on post-drive state
        let mut save_ns = Vec::with_capacity(archive_samples);
        let mut enc_ns = Vec::with_capacity(archive_samples);
        let mut dec_ns = Vec::with_capacity(archive_samples);
        let mut restore_ns = Vec::with_capacity(archive_samples);
        let mut roundtrip_fail = 0usize;
        let mut hash_mismatch = 0usize;
        let mut archive_bytes = 0usize;
        for _ in 0..archive_samples {
            let t0 = Instant::now();
            let snap = e.save();
            save_ns.push(t0.elapsed().as_nanos());
            let t1 = Instant::now();
            let bytes = encode_archive(&snap);
            enc_ns.push(t1.elapsed().as_nanos());
            archive_bytes = bytes.len();
            let t2 = Instant::now();
            let decoded = decode_archive(&bytes).expect("decode_archive");
            dec_ns.push(t2.elapsed().as_nanos());
            let t3 = Instant::now();
            let restored = if refs.is_empty() {
                Engine::restore(&decoded)
            } else {
                Engine::restore_with_extensions(&decoded, &refs)
            };
            restore_ns.push(t3.elapsed().as_nanos());
            match restored {
                Ok(r) => {
                    if r.state_hash() != e.state_hash() {
                        hash_mismatch += 1;
                    }
                }
                Err(_) => roundtrip_fail += 1,
            }
        }

        let b_st = stats(&build_ns);
        let cmd_st = stats(&run.cmd_ns);
        let plan_st = stats(&run.plan_ns);
        let sub_st = stats(&run.submit_ns);
        let sav_st = stats(&save_ns);
        let enc_st = stats(&enc_ns);
        let dec_st = stats(&dec_ns);
        let res_st = stats(&restore_ns);

        csv.push(&format!("sm_build_ns_{tier_id}"), &build_ns);
        csv.push(&format!("sm_cmd_ns_{tier_id}"), &run.cmd_ns);
        csv.push(&format!("sm_restore_ns_{tier_id}"), &restore_ns);

        emit_stats("source_matrix", tier_id, &cfg, "engine_build_ns", &b_st);
        emit_stats("source_matrix", tier_id, &cfg, "drive_cmd_ns", &cmd_st);
        emit_stats("source_matrix", tier_id, &cfg, "drive_plan_ns", &plan_st);
        emit_stats("source_matrix", tier_id, &cfg, "drive_submit_ns", &sub_st);
        emit_stats("source_matrix", tier_id, &cfg, "archive_save_ns", &sav_st);
        emit_stats("source_matrix", tier_id, &cfg, "archive_encode_ns", &enc_st);
        emit_stats("source_matrix", tier_id, &cfg, "archive_decode_ns", &dec_st);
        emit_stats("source_matrix", tier_id, &cfg, "archive_restore_ns", &res_st);

        emit_kv(
            "source_matrix",
            tier_id,
            &cfg,
            &[
                (
                    "result",
                    if roundtrip_fail == 0
                        && hash_mismatch == 0
                        && run.panics.is_empty()
                        && run.sim_faults == 0
                    {
                        "OK".into()
                    } else {
                        "FAIL".into()
                    },
                ),
                ("label", label.clone()),
                (
                    "dirs",
                    if dirs.is_empty() {
                        "-".into()
                    } else {
                        dirs.join("+")
                    },
                ),
                ("bridge_ns", bridge_ns.to_string()),
                ("cards", cards_len.to_string()),
                ("skills", skills_len.to_string()),
                ("extension_generals", generals_len.to_string()),
                ("ruleset_version", ruleset.to_string()),
                ("build_p50_ns", b_st.p50.to_string()),
                ("build_p95_ns", b_st.p95.to_string()),
                ("cmd_p50_ns", cmd_st.p50.to_string()),
                ("cmd_p95_ns", cmd_st.p95.to_string()),
                ("plan_p50_ns", plan_st.p50.to_string()),
                ("submit_p50_ns", sub_st.p50.to_string()),
                ("save_p50_ns", sav_st.p50.to_string()),
                ("encode_p50_ns", enc_st.p50.to_string()),
                ("decode_p50_ns", dec_st.p50.to_string()),
                ("restore_p50_ns", res_st.p50.to_string()),
                ("restore_p95_ns", res_st.p95.to_string()),
                ("archive_bytes", archive_bytes.to_string()),
                ("drive_steps", run.steps.to_string()),
                ("sim_faults", run.sim_faults.to_string()),
                ("plan_panics", run.panics.len().to_string()),
                ("roundtrip_failures", roundtrip_fail.to_string()),
                ("state_hash_mismatch", hash_mismatch.to_string()),
                ("state_hash", hex32(&e.state_hash())),
            ],
        );
    }
}
