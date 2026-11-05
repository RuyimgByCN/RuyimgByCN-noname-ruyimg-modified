//! R05-I 原字节锁链工厂与持久增量边界验收（合同 v3 §4/§7/§9 场景 4/5/6/11）。
//!
//! 覆盖：
//! 1. D/R/G 全窗口与逐微步（Before/Begin/Begin1/Begin2、R 逐张 held、G Filter/RecordHistory/Transfer/After、
//!    D End/After、跳过路径 Skipped、双枯竭转 detached_cards）的原字节幂等（save→encode→decode→restore→encode）、
//!    分片 abort（原引擎丢弃、仅用恢复出的新引擎继续）与终局哈希等价。
//! 2. 三固定 compound（tiqi 确认/奖励摸牌/符号询问及奖励 draw 后 owner 阵亡仍 forceDie 选符号；
//!    baoshu 成本询问/SkillLog/解链/翻面及按当前回合座次排序、进入 content 后 owner 阵亡不中断剩余目标；
//!    dcbihuo Source/End 询问与 TurnBegin 原子 take）的逐等待点原字节锁链。
//! 3. G.After 历史引用合法态：Transfer 后 held 已空、eligible 保留历史引用（含 After 期间牌再被弃置），
//!    恢复不把 eligible 重复算作 owner。
//! 4. 拒档负例：
//!    - 伪 Transfer（After 改回 Transfer、Filter/Transfer 伪造非 held 的 eligible、history_recorded 失配）；
//!    - 错 I（compound 绑定错 skill/player 或未派生对应角色 helper 的技能）；
//!    - 错锁/错版本（不同扩展集/标准目录恢复扩展档、不兼容存档版本）。

use engine_native::archive::{
    decode_archive, encode_archive, ArchiveError, ARCHIVE_FORMAT_VERSION,
};
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::event::{DrawGainProgress, DrawPhaseProgress, EventData};
use engine_native::rvm_ops::{dec_effects, enc_effects};
use engine_native::{
    ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot,
    OpenChoice, Phase, SnapshotError, StateSnapshot,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn pkg_root(dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir)
}

fn load_real_pack(dir: &str) -> ExtensionPack {
    let npack_json = std::fs::read_to_string(pkg_root(dir).join("package.npack.json"))
        .unwrap_or_else(|e| panic!("读 {dir}/package.npack.json 失败: {e}"));
    let npack: content_schema::ContentPackage =
        serde_json::from_str(&npack_json).expect("npack 反序列化");
    let decls = content_native_loader::bridge_vm_skill_decls(&npack)
        .unwrap_or_else(|e| panic!("桥接 {dir} 失败: {e:?}"))
        .decls;
    let chars = std::fs::read_to_string(pkg_root(dir).join("assets/characters.json"))
        .expect("读 characters.json");
    let generals = content_native_loader::bridge_generals_with_vm_package(&chars, &npack)
        .unwrap_or_else(|e| panic!("{dir} 武将装载失败: {e:?}"))
        .generals;
    let meta: serde_json::Value = serde_json::from_str(&chars).expect("characters.json 解析");
    let pack_name = meta["pack"]["name"].as_str().expect("pack.name");
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern(pack_name.to_string()),
            version: 1,
            description: engine_native::content::names::intern(
                meta["pack"]["description"].as_str().unwrap_or("").to_string(),
            ),
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

fn compound_module(name: &str, tag: u16) -> rule_vm::BytecodeModule {
    use rule_vm::bytecode::{assemble, caps, Op};
    let code = assemble(&[Op::HostSelf, Op::HostSkill, Op::HostEffect(tag), Op::Halt]);
    let caps_mask = caps::EMIT_EFFECTS;
    rule_vm::BytecodeModule {
        name: name.to_string(),
        logic_hash: rule_vm::BytecodeModule::compute_logic_hash(name, &code, 0, caps_mask, 0),
        code,
        local_count: 0,
        max_stack: 2,
        entry_pc: 0,
        caps_mask,
    }
}

fn retrigger(pkg: &mut content_schema::ContentPackage, tag: u16, events: &[&str]) {
    assert_eq!(pkg.trigger_index.len(), events.len());
    for (entry, event) in pkg.trigger_index.iter_mut().zip(events) {
        let name = entry.module.clone();
        let module = pkg
            .modules
            .iter_mut()
            .find(|m| m.name == name)
            .expect("trigger module in package");
        *module = compound_module(&name, tag);
        entry.event = event.to_string();
    }
}

fn compile_compound(
    package: &str,
    skill: &str,
    skill_id: u64,
    tag: u16,
    triggers: &[(&str, &str, &str)],
) -> content_schema::ContentPackage {
    let body: String = triggers
        .iter()
        .map(|(event, filter, _)| {
            format!("  trigger {event} {filter} {{\n    effect emit_note(0);\n  }}\n")
        })
        .collect();
    let source = format!("skill {skill}({skill_id}) {{\n{body}}}");
    let mut pkg = content_compiler::compile_source_named(package, &source)
        .unwrap_or_else(|e| panic!("编译 {skill} 失败: {e:?}"));
    let rewritten: Vec<&str> = triggers.iter().map(|(_, _, event)| *event).collect();
    retrigger(&mut pkg, tag, &rewritten);
    pkg.manifest.rule_vm_abi_version = 759;
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg
}

fn pack_of(general: &str, package: &str, pkg: &content_schema::ContentPackage) -> ExtensionPack {
    let decls = content_native_loader::bridge_vm_skill_decls(pkg)
        .unwrap_or_else(|e| panic!("桥接 {package} 失败: {e:?}"))
        .decls;
    let known = decls.iter().map(|d| d.name.clone()).collect();
    let granted: Vec<_> = decls
        .iter()
        .filter(|d| !d.name.contains("#r05:"))
        .map(|d| serde_json::json!({"name": d.name}))
        .collect();
    let generals = content_native_loader::bridge_generals_with_known(
        &serde_json::json!({
            "pack": {"name": package, "version": 1},
            "generals": [{"name": general, "hp": 3, "faction": "qun", "gender": "female",
                "skills": granted }]
        })
        .to_string(),
        &known,
    )
    .unwrap()
    .generals;
    let name = engine_native::content::names::intern(package.to_string());
    ExtensionPack {
        def: ExtensionDef {
            name,
            version: 1,
            description: "R05-I compound",
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

/// 锁链断言核心：对当前 Engine 执行 `save → encode → decode → restore → save → encode`，
/// 断言原字节完全幂等、公开视图与 choice context 一致，并丢弃原 Engine 返回新恢复的 Engine（分片 abort）。
fn roundtrip_and_replace(e: Engine, packs: &[&ExtensionPack]) -> Engine {
    let snap1 = e.save();
    let bytes1 = encode_archive(&snap1);
    let decoded = decode_archive(&bytes1).expect("合法快照字节解码");
    let restored = if packs.is_empty() {
        Engine::restore(&decoded).expect("标准局恢复")
    } else {
        Engine::restore_with_extensions(&decoded, packs).expect("扩展局恢复")
    };
    let mut snap2 = restored.save();
    // auth_generation 在每次 restore 自增 1（N11-a 令牌轮换），其余持久字节必须逐位相等
    assert_eq!(snap2.auth_generation, snap1.auth_generation + 1);
    snap2.auth_generation = snap1.auth_generation;
    let bytes2 = encode_archive(&snap2);
    assert_eq!(bytes1, bytes2, "save→restore→save 原字节必须幂等");
    assert_eq!(e.state_hash(), restored.state_hash(), "恢复前后规范哈希一致");
    assert_eq!(e.open_choice(), restored.open_choice(), "恢复前后 open_choice 一致");
    for seat in 0..e.state_core().players.len() as u8 {
        assert_eq!(
            e.view_for(seat),
            restored.view_for(seat),
            "恢复前后 seat {seat} 投影（含 r05_effects 与 choice.context）一致"
        );
    }
    // 分片 abort：显式 drop 原引擎，只用恢复出的新引擎继续
    drop(e);
    restored
}

fn default_answer(open: &OpenChoice) -> Vec<O> {
    if open.spec.options.contains(&O::Pass) {
        vec![O::Pass]
    } else if open.spec.allow_cancel {
        vec![]
    } else if open.spec.options.len() >= open.spec.min {
        open.spec.options.iter().take(open.spec.min).copied().collect()
    } else {
        panic!("无法默认应答: {:?}", open.spec)
    }
}

/// 1. D/R/G 全窗口与逐微步原字节锁链（time_slice=1 截停每个微步：
///    Before/Begin/Begin1/Begin2、R 逐张 held、G Filter/RecordHistory/Transfer/After、End/After）。
#[test]
fn r05i_drg_every_step_slice_byte_chain() {
    let mut direct = Engine::new(35001, 3);
    let mut chained = Engine::new(35001, 3);
    assert_eq!(direct.turn().phase, Phase::Judge);
    assert_eq!(chained.turn().phase, Phase::Judge);

    let dedup = next_dedup();
    // 基准线直跑完摸牌阶段
    let out_direct = direct.submit(GameCommand {
        actor: 0,
        expected_revision: direct.revision(),
        dedup_id: dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out_direct, CommandOutcome::Accepted { .. }));
    assert_eq!(direct.turn().phase, Phase::Draw);

    // 锁链线：设 time_slice=1，每推进 1 步做一次完整原字节往返并替换引擎
    chained.set_time_slice(Some(1));
    let mut outcome = chained.submit(GameCommand {
        actor: 0,
        expected_revision: chained.revision(),
        dedup_id: dedup,
        action: GameAction::NextPhase,
    });

    let mut seen_dp = Vec::new();
    let mut seen_held_lens = Vec::new();
    let mut seen_gp = Vec::new();
    let mut steps = 0usize;

    while matches!(outcome, CommandOutcome::Suspended) {
        steps += 1;
        assert!(steps < 128, "D/R/G 微步数超界");
        for f in &chained.save().resolution.stack {
            match &f.data {
                EventData::DrawPhase { progress, .. } => {
                    if seen_dp.last() != Some(progress) {
                        seen_dp.push(*progress);
                    }
                }
                EventData::Draw { held_cards, .. } => {
                    if seen_held_lens.last() != Some(&held_cards.len()) {
                        seen_held_lens.push(held_cards.len());
                    }
                }
                EventData::DrawGain { progress, .. } => {
                    if seen_gp.last() != Some(progress) {
                        seen_gp.push(*progress);
                    }
                }
                _ => {}
            }
        }
        chained = roundtrip_and_replace(chained, &[]);
        chained.set_time_slice(Some(1));
        outcome = chained.resume();
    }
    assert!(matches!(outcome, CommandOutcome::Accepted { .. }));
    assert!(
        seen_dp.contains(&DrawPhaseProgress::Before)
            && seen_dp.contains(&DrawPhaseProgress::Begin)
            && seen_dp.contains(&DrawPhaseProgress::Begin1)
            && seen_dp.contains(&DrawPhaseProgress::Begin2)
            && seen_dp.contains(&DrawPhaseProgress::MainWaiting)
            && seen_dp.contains(&DrawPhaseProgress::End)
            && seen_dp.contains(&DrawPhaseProgress::After),
        "必须历经 D 全部阶段: {seen_dp:?}"
    );
    assert!(
        seen_held_lens.contains(&1) && seen_held_lens.contains(&2),
        "必须历经 R 逐张 held (1, 2): {seen_held_lens:?}"
    );
    assert!(
        seen_gp.contains(&DrawGainProgress::Filter)
            && seen_gp.contains(&DrawGainProgress::RecordHistory)
            && seen_gp.contains(&DrawGainProgress::Transfer)
            && seen_gp.contains(&DrawGainProgress::After),
        "必须历经 G 全部阶段: {seen_gp:?}"
    );
    assert_eq!(
        direct.state_hash(),
        chained.state_hash(),
        "逐微步保存→恢复→继续终态与直跑完全同哈希"
    );
}

/// 2. 双枯竭转 detached_cards 与跳过路径（outcome=Skipped）的逐微步原字节锁链。
#[test]
fn r05i_drg_skipped_and_dual_exhausted_detached_slice_byte_chain() {
    // A) 双枯竭：牌堆剩 1 张、弃牌堆清空，摸 2 张时第 1 张进 held，第 2 张触发双枯竭 → held 转 detached_cards
    let mut e = Engine::new(35002, 3);
    // 把牌堆留 1 张，其余全部放入 seat2 手牌，弃牌堆清空（保证总牌数守恒且双枯竭）
    while e.state_core().deck.len() > 1 {
        let c = e.state_core_mut().deck.pop().unwrap();
        e.state_core_mut().player_mut(2).hand.push(c);
    }
    let disc = std::mem::take(&mut e.state_core_mut().discard);
    e.state_core_mut().player_mut(2).hand.extend(disc);

    e.set_time_slice(Some(1));
    let mut out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    while matches!(out, CommandOutcome::Suspended) {
        // 每一步（含 Exhausted 刚把 held 转入 detached_cards、Draw 帧尚未弹栈的瞬间）都必须通过 ownership 与原字节幂等！
        e = roundtrip_and_replace(e, &[]);
        e.set_time_slice(Some(1));
        out = e.resume();
    }
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "out={out:?}");
    assert_eq!(
        e.state_core().detached_cards.len(),
        1,
        "双枯竭时已抽 1 张转入 detached_cards"
    );
    let _ = roundtrip_and_replace(e, &[]);

    // B) 跳过路径（outcome=Skipped）：兵粮寸断非梅花判定跳过摸牌阶段，D 帧预置 Skipped 发 After 窗
    let mut skip_e = Engine::new(35003, 3);
    skip_e.state_core_mut().turn.skip_draw_phase = true;
    skip_e.set_time_slice(Some(1));
    let mut out_skip = skip_e.submit(GameCommand {
        actor: 0,
        expected_revision: skip_e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let mut saw_skipped = false;
    while matches!(out_skip, CommandOutcome::Suspended) {
        for f in &skip_e.save().resolution.stack {
            if let EventData::DrawPhase { outcome, .. } = &f.data {
                if *outcome == engine_native::event::DrawOutcome::Skipped {
                    saw_skipped = true;
                }
            }
        }
        skip_e = roundtrip_and_replace(skip_e, &[]);
        skip_e.set_time_slice(Some(1));
        out_skip = skip_e.resume();
    }
    assert!(saw_skipped, "必须捕获到 outcome=Skipped 的 D 帧");
}

/// 3. 三固定 compound 全询问点原字节锁链＋tiqi 奖励摸牌后 owner 阵亡仍 forceDie 选符号。
#[test]
fn r05i_three_compounds_every_choice_and_force_die_byte_chain() {
    let xianding = load_real_pack("xianding-upstream-noname");
    let sp2 = load_real_pack("sp2-upstream-noname");
    let packs = [&xianding, &sp2];

    // A) 直跑基准线 vs 每询问点 save→restore→save 锁链线（同剧本来回演练三技能全部询问点）
    let run_scenario = |do_roundtrip: bool| -> [u8; 32] {
        let mut e = Engine::new_with_extensions(35004, 3, &packs).unwrap();
        // seat0: sp2冯方(dcbihuo), seat1: xnd冯妤(tiqi+baoshu)
        e.assign_general_named(0, "sp2冯方").unwrap();
        e.assign_general_named(1, "xnd冯妤").unwrap();
        // 让 seat0 初始摸牌阶段 n != 2（给 seat0 预置 dcbihuo Current = +1 → n=3），触发 seat1 的 tiqi 两问
        let cur_def = e.catalog().skill_by_name("dcbihuo#r05:current").unwrap() as u32;
        e.state_core_mut().vm_aux.marks.insert((0, cur_def), 1);
        // 让 seat1 linked=true, face_down=true 以在 seat1 Prepare 触发 baoshu 的解链与翻面子结算
        e.state_core_mut().player_mut(1).linked = true;
        e.state_core_mut().player_mut(1).face_down = true;

        let mut step_guard = 0usize;
        let mut played_sha = false;
        loop {
            step_guard += 1;
            assert!(step_guard < 200, "剧本步数超界");
            if do_roundtrip {
                e = roundtrip_and_replace(e, &packs);
            }
            if let Some(open) = e.open_choice().cloned() {
                let sel = match open.spec.prompt {
                    "rvm_tiqi_confirm" => vec![O::Confirm],
                    "rvm_tiqi_sign" => vec![O::Tag(2)], // -1
                    "rvm_baoshu_cost" => vec![O::Player(0), O::Player(1)],
                    "rvm_dcbihuo_target" => vec![O::Player(1)],
                    _ => default_answer(&open),
                };
                let out = e.submit(GameCommand {
                    actor: open.observer,
                    expected_revision: e.revision(),
                    dedup_id: step_guard as u64,
                    action: GameAction::Answer {
                        choice_id: open.id,
                        selection: sel,
                    },
                });
                assert!(!matches!(out, CommandOutcome::Rejected(_)), "answer rejected for {}: {out:?}", open.spec.prompt);
                continue;
            }
            // 在 seat0 出牌阶段打出一张杀给 seat1 触发 dcbihuo Source，随后推进到 seat1 出牌阶段结束测试
            if e.turn().current == 0 && e.turn().phase == Phase::Play && !played_sha {
                played_sha = true;
                e.debug_clear_hand(1);
                let sha = e.debug_place(0, "sha");
                let out = e.submit(GameCommand {
                    actor: 0,
                    expected_revision: e.revision(),
                    dedup_id: step_guard as u64,
                    action: GameAction::PlayCard {
                        card: sha,
                        targets: vec![1],
                    },
                });
                assert!(!matches!(out, CommandOutcome::Rejected(_)));
                if e.open_choice().is_some() {
                    continue;
                }
            }
            if e.turn().current == 1 && e.turn().phase == Phase::Play {
                break;
            }
            let actor = e.turn().current;
            let out = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: step_guard as u64,
                action: GameAction::NextPhase,
            });
            assert!(!matches!(out, CommandOutcome::Rejected(_)));
        }
        e.state_hash()
    };

    let hash_direct = run_scenario(false);
    let hash_chained = run_scenario(true);
    assert_eq!(
        hash_direct, hash_chained,
        "三技能全询问点保存→恢复→继续的终局哈希必须与直跑一致"
    );

    // B) 合同 v3 §5 / §9.5：tiqi 奖励摸牌期间 owner 阵亡，仍须 forceDie 弹出 rvm_tiqi_sign 询问并完成写槽！
    let mut e_fd = Engine::new_with_extensions(35005, 3, &packs).unwrap();
    e_fd.assign_general_named(1, "xnd冯妤").unwrap();
    let cur_def = e_fd.catalog().skill_by_name("dcbihuo#r05:current").unwrap() as u32;
    e_fd.state_core_mut().vm_aux.marks.insert((0, cur_def), 1); // seat0 摸 3 张 → n=3, d=1
    while e_fd.open_choice().is_none() {
        let a = e_fd.turn().current;
        e_fd.submit(GameCommand {
            actor: a,
            expected_revision: e_fd.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    assert_eq!(e_fd.open_choice().unwrap().spec.prompt, "rvm_tiqi_confirm");
    // 用 time_slice=1 答复确认，使 tiqi 进入 stage=2（奖励 draw 子树运行中）
    e_fd.set_time_slice(Some(1));
    let cid = e_fd.open_choice().unwrap().id;
    let out = e_fd.submit(GameCommand {
        actor: 1,
        expected_revision: e_fd.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![O::Confirm],
        },
    });
    assert!(matches!(out, CommandOutcome::Suspended));
    // 模拟奖励 draw 子事件期间 owner(seat1) 阵亡（alive=false）
    e_fd.state_core_mut().player_mut(1).alive = false;
    while matches!(e_fd.resume(), CommandOutcome::Suspended) {}
    let open = e_fd
        .open_choice()
        .expect("奖励 draw 后 owner 阵亡仍须 forceDie 打开 rvm_tiqi_sign")
        .clone();
    assert_eq!(open.spec.prompt, "rvm_tiqi_sign");
    assert_eq!(open.observer, 1);
    // 在死后 forceDie 询问点同样验证 save→restore→save 原字节锁链！
    e_fd.set_time_slice(None);
    e_fd = roundtrip_and_replace(e_fd, &packs);
    let out = e_fd.submit(GameCommand {
        actor: 1,
        expected_revision: e_fd.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: open.id,
            selection: vec![O::Tag(1)],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let limit_def = e_fd.catalog().skill_by_name("tiqi#r05:limit").unwrap() as u32;
    assert_eq!(
        e_fd.state_core().vm_aux.marks.get(&(0, limit_def)).copied(),
        Some(1),
        "owner 死后选符号仍成功写入 target Limit 槽"
    );
}

/// 4. G.After 历史引用合法态（合同 v3 §4 & §7）：
///    Transfer 后 c 已移入 recipient.hand（或再被 After 子事件移入弃牌堆），held 已空、eligible 仍保留 [c]；
///    恢复校验不得把 G.After 的 eligible 再次算作持牌 owner。
#[test]
fn r05i_gain_after_history_reference_not_counted_as_owner() {
    let mut e = Engine::new(35006, 3);
    e.set_time_slice(Some(1));
    let mut out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let mut reached_after = false;
    while matches!(out, CommandOutcome::Suspended) {
        let snap = e.save();
        if snap.resolution.stack.iter().any(|f| {
            matches!(
                &f.data,
                EventData::DrawGain {
                    progress: DrawGainProgress::After,
                    eligible,
                    ..
                } if !eligible.is_empty()
            )
        }) {
            reached_after = true;
            break;
        }
        out = e.resume();
    }
    assert!(reached_after, "必须截获到 DrawGainProgress::After 时刻");

    // 1) 原样恢复（c 在 recipient.hand，Draw.held_cards 为空，DrawGain.eligible 非空）
    e = roundtrip_and_replace(e, &[]);

    // 2) 模拟 After 子技能把刚获得的其中一张牌从 recipient.hand 弃置入 discard（eligible 仍保留历史引用）
    let mut snap = e.save();
    let c = snap.core.players[0].hand.pop().expect("刚获得的牌在手牌");
    snap.core.discard.push(c);
    let bytes = encode_archive(&snap);
    let decoded = decode_archive(&bytes).expect("After 弃牌后解码");
    let restored = Engine::restore(&decoded)
        .expect("G.After 的 eligible 仅作历史引用，牌已移入弃牌堆仍须合法恢复");
    assert_eq!(encode_archive(&snap), {
        let mut s2 = restored.save();
        s2.auth_generation = snap.auth_generation;
        encode_archive(&s2)
    });
}

/// 5. 拒档负例 A：伪 Transfer（把 After 改回 Transfer 但牌已离 held、或 Filter/Transfer 伪造非 held 的 eligible）。
#[test]
fn r05i_negative_tamper_forged_transfer_and_eligible_rejected() {
    let mut e = Engine::new(35007, 3);
    e.set_time_slice(Some(1));
    let mut out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let mut snap_filter: Option<StateSnapshot> = None;
    let mut snap_after: Option<StateSnapshot> = None;
    while matches!(out, CommandOutcome::Suspended) {
        let s = e.save();
        for f in &s.resolution.stack {
            if let EventData::DrawGain { progress, .. } = &f.data {
                if *progress == DrawGainProgress::Filter && snap_filter.is_none() {
                    snap_filter = Some(s.clone());
                }
                if *progress == DrawGainProgress::After && snap_after.is_none() {
                    snap_after = Some(s.clone());
                }
            }
        }
        out = e.resume();
    }

    // 负例 1：在 After 态（牌已入 recipient.hand，R.held 已清空）把 progress 篡改回 Transfer → 必须拒档！
    let mut bad_transfer = snap_after.expect("captured After");
    for f in &mut bad_transfer.resolution.stack {
        if let EventData::DrawGain { progress, .. } = &mut f.data {
            *progress = DrawGainProgress::Transfer;
        }
    }
    let res1 = Engine::restore(&bad_transfer);
    assert!(
        res1.is_err(),
        "把已转移的 G.After 篡改回 Transfer（牌已不在 R.held）必须拒档"
    );

    // 负例 2：在 Filter 态把 eligible 篡改为一张不在 R.held 中的牌（例如牌库顶牌）→ 必须拒档！
    let mut bad_eligible = snap_filter.expect("captured Filter");
    let non_held_card = bad_eligible.core.deck[0];
    for f in &mut bad_eligible.resolution.stack {
        if let EventData::DrawGain { eligible, .. } = &mut f.data {
            eligible[0] = non_held_card;
        }
    }
    let res2 = Engine::restore(&bad_eligible);
    assert!(
        res2.is_err(),
        "G.Filter 的 eligible 含不在 R.held 中的牌必须拒档"
    );
}

/// 6. 拒档负例 B：错 I（compound 绑定错 skill ID / 错 player ID / 未派生对应角色 helper 的技能）。
#[test]
fn r05i_negative_tamper_wrong_compound_skill_rejected() {
    let xianding = load_real_pack("xianding-upstream-noname");
    let packs = [&xianding];
    let mut e = Engine::new_with_extensions(35008, 3, &packs).unwrap();
    e.assign_general_named(1, "xnd冯妤").unwrap();
    e.state_core_mut().turn.skip_draw_phase = true;
    while e.open_choice().is_none() {
        let a = e.turn().current;
        e.submit(GameCommand {
            actor: a,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_tiqi_confirm");
    let base_snap = e.save();

    // 负例 1：篡改 commit_pending 内 ResolveTiqi 的 skill ID 为另一个目录技能（错 I）
    let mut bad_skill = base_snap.clone();
    let mut patched = false;
    for park in &mut bad_skill.resolution.parks {
        if let MachineSnapshot::Rvm { commit_pending, .. } = &mut park.machine {
            if let Some(mut effs) = dec_effects(commit_pending) {
                for eff in &mut effs {
                    if let rule_api::EffectRequest::ResolveTiqi { skill, .. } = eff {
                        *skill = rule_api::SkillId::new(skill.raw() ^ 1);
                        patched = true;
                    }
                }
                commit_pending.clear();
                enc_effects(commit_pending, &effs);
            }
        }
    }
    assert!(patched, "必须命中 ResolveTiqi 效果");
    assert!(
        matches!(
            Engine::restore_with_extensions(&bad_skill, &packs),
            Err(SnapshotError::UnknownMachine)
        ),
        "compound 绑定错 skill ID（错 I）必须以 UnknownMachine 拒档"
    );

    // 负例 2：篡改 commit_pending 内 ResolveTiqi 的 player ID 为非 owner
    // （不改 open_choice.observer 被 open_choice_spec_mismatch 拦截；即便同步伪造 observer 亦被 UnknownMachine 拦截）
    let mut bad_player = base_snap;
    for park in &mut bad_player.resolution.parks {
        if let MachineSnapshot::Rvm { commit_pending, .. } = &mut park.machine {
            if let Some(mut effs) = dec_effects(commit_pending) {
                for eff in &mut effs {
                    if let rule_api::EffectRequest::ResolveTiqi { player, .. } = eff {
                        *player = rule_api::PlayerId::new(1); // seat0 != owner(seat1)
                    }
                }
                commit_pending.clear();
                enc_effects(commit_pending, &effs);
            }
        }
    }
    assert!(
        Engine::restore_with_extensions(&bad_player, &packs).is_err(),
        "仅篡改 compound player ID 必须拒档"
    );
    bad_player.resolution.open_choice.as_mut().unwrap().observer = 0;
    assert!(
        matches!(
            Engine::restore_with_extensions(&bad_player, &packs),
            Err(SnapshotError::UnknownMachine)
        ),
        "即便同步伪造 open_choice.observer，compound 绑定错 player ID 仍须以 UnknownMachine 拒档"
    );
}

/// 7. 拒档负例 C：错锁/版本不符（不同包集或标准目录恢复扩展档、错误存档格式版本）。
#[test]
fn r05i_negative_tamper_wrong_lock_and_version_rejected() {
    let xianding = load_real_pack("xianding-upstream-noname");
    let sp2 = load_real_pack("sp2-upstream-noname");
    let e = Engine::new_with_extensions(35009, 3, &[&xianding]).unwrap();
    let snap = e.save();

    // 用标准目录或另一扩展包恢复 → RuleVersionMismatch
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ));
    assert!(matches!(
        Engine::restore_with_extensions(&snap, &[&sp2]),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ));

    // 篡改字节流中的存档格式版本为 106 → UnsupportedFormatVersion(106)
    let mut bytes = encode_archive(&snap);
    assert_eq!(
        u16::from_le_bytes([bytes[4], bytes[5]]),
        ARCHIVE_FORMAT_VERSION
    );
    bytes[4..6].copy_from_slice(&106u16.to_le_bytes());
    assert!(matches!(
        decode_archive(&bytes),
        Err(ArchiveError::UnsupportedFormatVersion(106))
    ));
}

/// 8. 宝梳（baoshu）按当前回合角色座次排序目标、且进入 content 后 owner 阵亡不中断剩余目标（合同 v3 §5 & §9.6）。
#[test]
fn r05i_baoshu_seat_order_from_turn_player_and_post_entry_owner_death() {
    let pkg = compile_compound(
        "r05i-baoshu",
        "r05i_baoshu_test",
        998100001,
        782,
        &[("phase_prepare", "owner_is_target", "phase_prepare")],
    );
    let pack = pack_of("baoshu_tester", "r05i-baoshu", &pkg);

    let mut e = Engine::new_with_extensions(35010, 3, &[&pack]).unwrap();
    e.assign_general_named(1, "baoshu_tester").unwrap();
    // 推进到 seat1 的准备阶段（rvm_baoshu_cost 打开）
    let mut guard = 0usize;
    while e.open_choice().map(|c| c.spec.prompt) != Some("rvm_baoshu_cost") {
        guard += 1;
        assert!(guard < 64, "推进到 seat1 准备阶段超预算");
        if let Some(open) = e.open_choice().cloned() {
            let sel = default_answer(&open);
            e.submit(GameCommand {
                actor: open.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: open.id,
                    selection: sel,
                },
            });
        } else {
            let a = e.turn().current;
            e.submit(GameCommand {
                actor: a,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            });
        }
    }
    // seat1 回合中选 [0, 1, 2]：按当前回合座次（seat1 起算）顺序应为 [1, 2, 0]（对应 +1 编码 [2, 3, 1]）！
    e.set_time_slice(Some(1));
    let cid = e.open_choice().unwrap().id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![O::Player(0), O::Player(1), O::Player(2)],
        },
    });
    assert!(matches!(out, CommandOutcome::Suspended));
    // 推进到 baoshu 进入 stage >= 4（content 逐目标循环）后，模拟 owner(seat1) 在首个目标子结算中阵亡
    e.state_core_mut().player_mut(1).linked = true;
    let mut step_guard = 0usize;
    while matches!(e.resume(), CommandOutcome::Suspended) {
        step_guard += 1;
        assert!(step_guard < 128, "baoshu resume 超预算");
        let mut in_stage_5 = false;
        for p in &e.save().resolution.parks {
            if let MachineSnapshot::Rvm { commit_state, .. } = &p.machine {
                if commit_state.first() == Some(&5) {
                    // 验证 targets 顺序为 [1+1, 2+1, 0+1] = [2, 3, 1]
                    assert_eq!(
                        &commit_state[4..],
                        &[2, 3, 1],
                        "targets 必须按当前回合角色座次排序"
                    );
                    in_stage_5 = true;
                }
            }
        }
        if in_stage_5 {
            e.state_core_mut().player_mut(1).alive = false;
            break;
        }
    }
    let mut finish_guard = 0usize;
    while matches!(e.resume(), CommandOutcome::Suspended) {
        finish_guard += 1;
        assert!(finish_guard < 128, "baoshu 收尾 resume 超预算");
    }
    let combs_def = e
        .catalog()
        .skill_by_name("r05i_baoshu_test#r05:combs")
        .unwrap() as u32;
    // seat1 已死不授标，但后续存活目标 seat2、seat0 仍须完成 Combs 授标（进入 content 后不因 owner 阵亡中断）！
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(2, combs_def)).copied(),
        Some(1),
        "进入 content 后 owner 阵亡不中断后续存活目标 seat2"
    );
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(0, combs_def)).copied(),
        Some(1),
        "进入 content 后 owner 阵亡不中断后续存活目标 seat0"
    );
}
