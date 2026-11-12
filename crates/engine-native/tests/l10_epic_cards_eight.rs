//! L10 史诗卡牌八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/epic-cards`。功能面为两个
//! 规则技能（`reguhuo` 蛊惑 / `rangjie` 让节，宿主行为 tag 515/516）；配置面
//! 走 `content-native-loader::epic_cards`（展示面开关 + 十常侍面四态依赖判定）；
//! 十常侍面因依赖 `k_danggu`/`k_mowang`（本仓未定义）为**结构性阻塞**，登记差异。

use std::path::PathBuf;

use content_native_loader::{
    parse_epic_config, ShichangshiPlan, EXTERNAL_SKILLS, OVERRIDE_CHARACTER, REQUIRED_EXTENSION,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn upstream_extension() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js");
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("读上游 {}", path.display()))
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn epic_pack() -> engine_native::content::extension::ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/cards.json"),
    )
    .expect("读史诗卡牌包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

fn engine(seed: u64) -> Engine {
    Engine::new_with_extensions(seed, 4, &[&epic_pack()]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
}

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}

/// L10 八项。
#[test]
fn l10_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    // ---- 上游回读（功能面存在性）----
    let upstream = upstream_extension();
    assert!(upstream.contains("lib.skill.reguhuo"));
    assert!(upstream.contains("lib.skill.rangjie"));
    assert!(upstream.contains("lib.config['extension_十周年UI_enable']"));
    assert!(upstream.contains("k_danggu"));
    assert!(upstream.contains("k_mowang"));

    // ---- 功能：蛊惑（声明牌种使用）----
    let mut e = engine(140001);
    e.debug_grant(0, "reguhuo");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    e.debug_clear_hand(0);
    let card = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, card);
    e.debug_set_hp(0, 2);
    let hp0 = e.debug_hp(0);
    let def = e.catalog().skill_by_name("reguhuo").expect("蛊惑");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "reguhuo_card", vec![ChoiceOption::Card(card)]),
        "蛊惑选牌窗"
    );
    // 声明【桃】（下标 4）
    assert!(
        answer(&mut e, "reguhuo_name", vec![ChoiceOption::Tag(4)]),
        "蛊惑声明窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 + 1, "蛊惑：声明桃回复 1");

    // ---- 功能：让节（受伤后获得指定类型牌）----
    let mut e2 = engine(140002);
    e2.debug_grant(0, "rangjie");
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let out = e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e2, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "让节确认门"
    );
    assert!(
        answer(&mut e2, "rangjie_choice", vec![ChoiceOption::Tag(0)]),
        "让节选择窗"
    );
    assert!(
        answer(&mut e2, "rangjie_type", vec![ChoiceOption::Tag(0)]),
        "让节类型窗"
    );
    drain(&mut e2);
    assert_eq!(e2.debug_hand(0).len(), 2, "让节：摸 1 + 得 1");

    // ---- 配置：展示面开关 + 十常侍面依赖判定 ----
    let stored_config = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/config.json"),
    )
    .expect("读配置");
    let cfg = parse_epic_config(&stored_config).expect("解析配置");
    assert_eq!(cfg.upstream_commit, "8485dbb29ff16f9b258a766b1a17ef75f883aade");
    assert!(cfg.card_effects && cfg.skill_effects);
    assert!(!cfg.shichangshi, "默认关闭十常侍面");
    let pres = cfg.presentation();
    assert!(pres.card_effects && pres.skill_effects);
    assert!(!stored_config.contains("eval(") && !stored_config.contains("fetch("));
    let mut cfg_on = cfg.clone();
    cfg_on.shichangshi = true;
    assert_eq!(
        cfg_on.shichangshi_plan(false, |_| true),
        ShichangshiPlan::MissingCharacter(OVERRIDE_CHARACTER.to_string())
    );
    assert_eq!(
        cfg_on.shichangshi_plan(true, |_| false),
        ShichangshiPlan::MissingSkill(EXTERNAL_SKILLS[0].to_string())
    );
    match cfg_on.shichangshi_plan(true, |_| true) {
        ShichangshiPlan::Apply {
            required_extension, ..
        } => assert_eq!(required_extension, REQUIRED_EXTENSION),
        other => panic!("应为 Apply：{other:?}"),
    }

    // ---- 组合：与 swd 包同时装载（两包技能均可授予）----
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包");
    let swd = content_native_loader::bridge_pack(&swd_json).expect("桥接 swd").pack;
    let both = Engine::new_with_extensions(140003, 4, &[&epic_pack(), &swd]).expect("双包建局");
    let mut both = both;
    both.debug_grant(0, "reguhuo");
    both.debug_grant(0, "rangjie");
    both.debug_grant(0, "jilve");
    for name in ["reguhuo", "rangjie", "jilve"] {
        let d = both.catalog().skill_by_name(name).expect(name);
        assert!(
            both.state_core().skills.contains_key(&(0, d)),
            "组合：{name} 在册"
        );
    }

    // ---- 保存恢复：带包存档往返 ----
    let mut e3 = engine(140004);
    e3.debug_grant(0, "rangjie");
    e3.debug_set_hp(0, 2);
    let snap = e3.save();
    let restored = Engine::restore_with_extensions(&snap, &[&epic_pack()]).expect("恢复");
    let d = restored.catalog().skill_by_name("rangjie").expect("rangjie");
    assert!(
        restored.state_core().skills.contains_key(&(0, d)),
        "保存恢复：技能实例保留"
    );
    assert_eq!(restored.state_core().player(0).hp, 2);

    // ---- 安装：真实 import ----
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "epic-cards" && item.namespace == "upstream-liuli")
        .expect("史诗卡牌已安装");
    assert_eq!(meta.version, "1.0.0");
    let npack_bytes =
        std::fs::read(repo_extensions_root().join("epic-cards-upstream-liuli/package.npack.json"))
            .unwrap();
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );

    // ---- 依赖：本包无 manifest 依赖；十常侍面依赖为内容级（已登记）----
    // 商店目录布局：`<root>/packages/<namespace>__<package_id>/<version>`
    let installed_dir = store
        .root()
        .join("packages")
        .join("upstream-liuli__epic-cards")
        .join("1.0.0");
    let installed = content_schema::ContentPackage::from_json_bytes(
        &std::fs::read(installed_dir.join("package.json")).unwrap(),
    )
    .expect("读商店包");
    assert!(installed.manifest.dependencies.is_empty());
    assert_eq!(installed.skill_index.len(), 2, "两个技能条目");
    let names: Vec<&str> = installed
        .skill_index
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert!(names.contains(&"reguhuo") && names.contains(&"rangjie"));
    assert!(installed.card_index.is_empty() && installed.character_index.is_empty());

    // ---- 保存恢复（商店）：重开商店仍 1.0.0 ----
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        reopened.current_lock().expect("lock").lock_hash,
        lock_before
    );

    // ---- 安装升级：2.0.0 覆盖 + rollback ----
    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &options(),
            &installed_dir,
        )
        .expect("安装 2.0.0");
    assert_eq!(
        store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("epic-cards"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default(),
        "2.0.0"
    );
    store.rollback().expect("rollback");
    assert_eq!(
        store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("epic-cards"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default(),
        "1.0.0"
    );

    // ---- 依赖（错误路径）：未安装版本 NotFound ----
    match store.activate_version("upstream-liuli", "epic-cards", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }

    // ---- 规则差异：蛊惑声明面（Tag 表长度）与让节「牌堆取首张」----
    let mut e4 = engine(140005);
    e4.debug_grant(0, "reguhuo");
    for _ in 0..12 {
        if e4.turn().current == 0 && e4.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e4);
        drain(&mut e4);
    }
    let def = e4.catalog().skill_by_name("reguhuo").expect("reguhuo");
    let out = e4.submit(GameCommand {
        actor: 0,
        expected_revision: e4.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e4, "reguhuo_card", Vec::new()),
        "蛊惑选牌窗（取消）"
    );
    drain(&mut e4);

    // ---- 功能：党锢（k_danggu）与殁亡（k_mowang）宿主行为（tag 549/550）----
    let mut e5 = engine(140006);
    e5.debug_grant(0, "k_danggu");
    e5.debug_grant(0, "k_mowang");
    next_phase(&mut e5);
    let c5 = e5.open_choice().expect("k_danggu_partner").clone();
    assert_eq!(c5.spec.prompt, "k_danggu_partner");
    assert!(answer(
        &mut e5,
        "k_danggu_partner",
        vec![c5.spec.options[0]]
    ));
    drain(&mut e5);
    assert_eq!(
        e5.state_core()
            .vm_aux
            .flags
            .get(&(0, "k_danggu_roster_mask".to_string()))
            .copied()
            .unwrap_or(0)
            .count_ones(),
        8
    );
}

fn submit_cmd(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
        drain(e);
    }
    panic!("未推进到座位 {seat} 出牌阶段");
}

fn assert_archive_roundtrip_and_answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    use engine_native::archive::{decode_archive, encode_archive};
    let pack = epic_pack();
    let c = e.open_choice().cloned().expect("pending choice open");
    let log_before = e.log().len();
    let raw = encode_archive(&e.save());
    let decoded = decode_archive(&raw).expect("decode archive");
    let mut restored =
        Engine::restore_with_extensions(&decoded, &[&pack]).expect("restore archive");
    assert_eq!(e.state_hash(), restored.state_hash(), "恢复前 state_hash 一致");
    let out_orig = submit_cmd(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: selection.clone(),
        },
    );
    let out_rest = submit_cmd(
        &mut restored,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert_eq!(
        std::mem::discriminant(&out_orig),
        std::mem::discriminant(&out_rest),
        "原局与恢复局提交结果一致"
    );
    assert_eq!(e.state_hash(), restored.state_hash(), "应答后 state_hash 一致");
    assert_eq!(&e.log()[log_before..], restored.log(), "应答后事件序列一致");
}

fn assert_ai_pending_choice_no_leak(e: &Engine) -> Vec<ChoiceOption> {
    use engine_native::ai::BoundedPlanner;
    let pack = epic_pack();
    let c = e.open_choice().expect("pending choice for AI");
    let obs = c.observer;
    let planner = BoundedPlanner::new(16);
    let cmd_a = e.plan_action(obs, &planner);
    let GameAction::Answer {
        choice_id,
        selection: sel_a,
    } = cmd_a.action
    else {
        panic!("expected GameAction::Answer from AI, got {:?}", cmd_a.action);
    };
    assert_eq!(choice_id, c.id);

    let mut snap_b = e.save();
    snap_b.core.deck.reverse();
    if let engine_native::GameMode::Identity { ref mut roles } = snap_b.core.mode {
        let hidden: Vec<usize> = (1..roles.len()).filter(|&s| s != obs as usize).collect();
        if hidden.len() >= 2 {
            roles.swap(hidden[0], hidden[1]);
        }
    }
    let e_b = Engine::restore_with_extensions(&snap_b, &[&pack]).expect("restore hidden flip");
    let cmd_b = e_b.plan_action(obs, &planner);
    let GameAction::Answer {
        selection: sel_b, ..
    } = cmd_b.action
    else {
        panic!("expected Answer");
    };
    assert_eq!(sel_a, sel_b, "AI 决策不得读取未公开暗信息 (prompt={})", c.spec.prompt);
    sel_a
}

/// 蛊惑（`reguhuo`）六维真实对局测试：①时机与否定 ②筛选与上限 ③选择分支 ④多步状态变更 ⑤挂起窗口存档恢复 ⑥AI 合法规划。
#[test]
fn l10_reguhuo_six_dimension_real_match() {
    let mut e = engine(140101);
    e.debug_grant(0, "reguhuo");
    let def = e.catalog().skill_by_name("reguhuo").expect("reguhuo");

    // ① 触发时机与否定条件：非出牌阶段拒绝发动
    assert_ne!(e.turn().phase, Phase::Play);
    assert!(matches!(
        submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));

    advance_to_play(&mut e, 0);
    // 非持技者 / 非当前回合角色拒绝发动
    assert!(matches!(
        submit_cmd(&mut e, 1, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));
    // 空手牌时拒绝发动
    e.debug_clear_hand(0);
    assert!(matches!(
        submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));

    // ② 选牌/牌名/目标筛选与上限 + ⑤ 挂起窗口存档恢复 + ⑥ AI 规划
    let bluff_card = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, bluff_card);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    assert!(matches!(
        submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let c_card = e.open_choice().cloned().expect("reguhuo_card");
    assert_eq!(c_card.spec.prompt, "reguhuo_card");
    assert_eq!(c_card.spec.options, vec![ChoiceOption::Card(bluff_card)]);
    assert_eq!((c_card.spec.min, c_card.spec.max, c_card.spec.allow_cancel), (1, 1, true));
    let _ = assert_ai_pending_choice_no_leak(&e);
    assert_archive_roundtrip_and_answer(&mut e, vec![ChoiceOption::Card(bluff_card)]);

    let c_name = e.open_choice().cloned().expect("reguhuo_name");
    assert_eq!(c_name.spec.prompt, "reguhuo_name");
    assert!(c_name.spec.options.contains(&ChoiceOption::Tag(0))); // 杀
    assert!(c_name.spec.options.contains(&ChoiceOption::Tag(4))); // 桃
    let _ = assert_ai_pending_choice_no_leak(&e);
    // 伪装声明【杀】（Tag(0)，载体实为【桃】）
    assert_archive_roundtrip_and_answer(&mut e, vec![ChoiceOption::Tag(0)]);

    let c_tgt = e.open_choice().cloned().expect("reguhuo_targets");
    assert_eq!(c_tgt.spec.prompt, "reguhuo_targets");
    assert!(!c_tgt.spec.options.contains(&ChoiceOption::Player(0)), "不可杀自己");
    assert!(c_tgt.spec.options.contains(&ChoiceOption::Player(1)));
    let _ = assert_ai_pending_choice_no_leak(&e);

    // ③ 分支对比：目标无闪放弃响应（命中扣血）vs 目标出闪抵消
    let snap_before_tgt = e.save();
    let hp1_before = e.debug_hp(1);
    assert_archive_roundtrip_and_answer(&mut e, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_before - 1, "分支 A：伪装杀未被抵消，造成 1 点伤害");

    // 同一场再次发动（每回合限一次）必须被拒绝
    let extra = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, extra);
    assert!(matches!(
        submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));

    // 分支 B：目标持有【闪】并响应抵消蛊惑声明的【杀】
    let mut e_dodge = Engine::restore_with_extensions(&snap_before_tgt, &[&epic_pack()]).unwrap();
    let shan = e_dodge.debug_place(1, "shan");
    e_dodge.debug_give_hand_card(1, 1, shan);
    let c_tgt_b = e_dodge.open_choice().cloned().unwrap();
    let _ = submit_cmd(
        &mut e_dodge,
        0,
        GameAction::Answer {
            choice_id: c_tgt_b.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c_shan = e_dodge.open_choice().cloned().expect("目标闪响应窗");
    assert_eq!(c_shan.observer, 1);
    let _ = submit_cmd(
        &mut e_dodge,
        1,
        GameAction::Answer {
            choice_id: c_shan.id,
            selection: vec![ChoiceOption::Card(shan)],
        },
    );
    drain(&mut e_dodge);
    assert_eq!(e_dodge.debug_hp(1), hp1_before, "分支 B：目标出闪抵消蛊惑杀，体力不变");

    // ④ 跨回合多步状态推进：推进到下回合出牌阶段，每回合限一次重置，声明【桃】回复体力
    next_phase(&mut e);
    drain(&mut e);
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 2);
    let _ = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(answer(&mut e, "reguhuo_card", vec![ChoiceOption::Card(extra)]));
    assert!(answer(&mut e, "reguhuo_name", vec![ChoiceOption::Tag(4)]));
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 3, "跨回合重置后蛊惑声明桃回复 1 点体力");
}

/// 让节（`rangjie`）六维真实对局测试：①受伤时机与否定 ②模式/类型/移动筛选与上限 ③三分支对比 ④2点伤害连续两轮多步状态变更 ⑤挂起窗口存档恢复 ⑥AI 合法规划。
#[test]
fn l10_rangjie_six_dimension_real_match() {
    let mut e = engine(140201);
    e.debug_grant(0, "rangjie");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 触发时机与否定条件：非持技者受伤（1号位受1伤）不触发 0号位让节；0点伤害不触发
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "他人受伤不触发让节");
    let _ = e.debug_deal_damage(Some(1), 0, 0, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "0点伤害不触发让节");

    // 确认门选择取消（Vec::new()）放弃发动：不进入 rangjie_choice
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    assert!(answer(&mut e, "trigger_confirm", Vec::new()));
    assert!(e.open_choice().is_none(), "放弃确认后不打开让节选择窗");
    assert!(e.debug_hand(0).is_empty());

    // ② 筛选与上限（场上无装备/判定牌时 Tag(1) 移动分支被过滤，仅 Tag(0)/Tag(2)）
    e.debug_set_hp(0, 4);
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]));
    let c_no_eq = e.open_choice().cloned().expect("rangjie_choice without field cards");
    assert_eq!(c_no_eq.spec.prompt, "rangjie_choice");
    assert_eq!(
        c_no_eq.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(2)],
        "场上无装备/判定牌时仅提供 Tag(0) 获得牌与 Tag(2) 取消"
    );
    // ③ 分支 Tag(2)（取消）：不摸牌、不移动
    let _ = assert_ai_pending_choice_no_leak(&e);
    assert_archive_roundtrip_and_answer(&mut e, vec![ChoiceOption::Tag(2)]);
    assert!(e.open_choice().is_none());
    assert!(e.debug_hand(0).is_empty(), "Tag(2) 取消分支不获得牌");

    // ④ 2点伤害触发连续 2 次让节 + ②③⑤⑥ 移动场上装备分支（Tag(1)）与获得锦囊牌分支（Tag(0)->Tag(1)）
    let mut e2 = engine(140202);
    e2.debug_grant(0, "rangjie");
    advance_to_play(&mut e2, 1);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let weapon = e2.debug_place(1, "zhuge");
    e2.debug_give_hand_card(1, 1, weapon);
    let _ = submit_cmd(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: weapon,
            targets: Vec::new(),
        },
    );
    drain(&mut e2);
    assert!(e2.state_core().player(1).equips.contains(&weapon), "1号位已装备诸葛连弩");

    e2.debug_set_hp(0, 4);
    let _ = e2.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(answer(&mut e2, "trigger_confirm", vec![ChoiceOption::Confirm]));

    // 第 1 次让节：场上已有 1号位装备，options 含 Tag(0), Tag(1), Tag(2)
    let c1 = e2.open_choice().cloned().expect("第1次 rangjie_choice");
    assert_eq!(
        c1.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)]
    );
    let _ = assert_ai_pending_choice_no_leak(&e2);
    // 选 Tag(1)（摸 1 张并移动场上牌）
    assert_archive_roundtrip_and_answer(&mut e2, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e2.debug_hand(0).len(), 1, "Tag(1) 先摸 1 张牌");

    let c_move = e2.open_choice().cloned().expect("rangjie_move_card");
    assert_eq!(c_move.spec.prompt, "rangjie_move_card");
    assert_eq!(c_move.spec.options, vec![ChoiceOption::Card(weapon)]);
    let _ = assert_ai_pending_choice_no_leak(&e2);
    assert_archive_roundtrip_and_answer(&mut e2, vec![ChoiceOption::Card(weapon)]);

    let c_dest = e2.open_choice().cloned().expect("rangjie_move_target");
    assert_eq!(c_dest.spec.prompt, "rangjie_move_target");
    assert_eq!(c_dest.spec.options.len(), 4);
    let _ = assert_ai_pending_choice_no_leak(&e2);
    assert_archive_roundtrip_and_answer(&mut e2, vec![ChoiceOption::Player(0)]);
    assert!(!e2.state_core().player(1).equips.contains(&weapon), "1号位武器已移走");
    assert!(e2.state_core().player(0).equips.contains(&weapon), "0号位获得该武器入装备区");

    // 第 2 次让节（因受到 2 点伤害自动续行第 2 次）：选 Tag(0) 摸 1 + 选 Tag(1) 检索锦囊牌
    let c2 = e2.open_choice().cloned().expect("第2次 rangjie_choice");
    assert_eq!(c2.spec.prompt, "rangjie_choice");
    assert_archive_roundtrip_and_answer(&mut e2, vec![ChoiceOption::Tag(0)]);

    let c_type = e2.open_choice().cloned().expect("rangjie_type");
    assert_eq!(c_type.spec.prompt, "rangjie_type");
    assert_eq!(
        c_type.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)]
    );
    let _ = assert_ai_pending_choice_no_leak(&e2);
    assert_archive_roundtrip_and_answer(&mut e2, vec![ChoiceOption::Tag(1)]);
    drain(&mut e2);
    assert_eq!(e2.debug_hand(0).len(), 3, "第1次摸1 + 第2次摸1 + 检索锦囊1 = 3张手牌");
    let last_card = *e2.debug_hand(0).last().unwrap();
    let kind = e2.catalog().card_kind(last_card);
    assert!(!kind.is_basic_kind() && !kind.is_equip(), "Tag(1) 检索到的牌必为锦囊牌");
}

/// 党锢（`k_danggu`）+ 殁亡（`k_mowang`）+ 10 项配置六维真实对局测试。
#[test]
fn l10_kdanggu_kmowang_and_ten_configs_six_dimension_real_match() {
    use content_native_loader::EpicError;

    // ---- 10 项配置与四态依赖完整核验 ----
    let bad_scale = r#"{"source":"s","upstream_path":"p","upstream_commit":"c","skillEffectsScale":12.5}"#;
    assert_eq!(
        parse_epic_config(bad_scale),
        Err(EpicError::BadField("skillEffectsScale".into()))
    );
    let custom_json = r#"{
        "source":"noname-liuli","upstream_path":"extension/史诗卡牌","upstream_commit":"8485dbb",
        "cardEffects":false,"skillEffects":true,"skillEffectsScale":2.5,
        "cardEmoji":true,"cardFold":true,"cardPhantom":true,
        "meihuaskills":true,"shichangshi":true,"skinEffects":true
    }"#;
    let cfg = parse_epic_config(custom_json).expect("解析 10 项配置");
    assert!(!cfg.card_effects && cfg.skill_effects && (cfg.skill_effects_scale - 2.5).abs() < 1e-6);
    assert!(cfg.card_emoji && cfg.card_fold && cfg.card_phantom && cfg.meihua_skills && cfg.shichangshi && cfg.skin_effects);
    let pres = cfg.presentation();
    assert!(!pres.card_effects && pres.skill_effects && pres.card_emoji && pres.card_fold && pres.card_phantom && pres.skin_effects);
    assert_eq!(
        cfg.shichangshi_plan(true, |s| s != "k_mowang"),
        ShichangshiPlan::MissingSkill("k_mowang".into())
    );

    // ---- ② 党锢 conflictMap（3<->5, 7<->8, 9<->[3,5,7,8]）与高望(9)保底纳入 ----
    for mask in [
        (1u32 << 0) | (1 << 3) | (1 << 5) | (1 << 9),
        (1u32 << 0) | (1 << 7) | (1 << 8) | (1 << 9),
    ] {
        for seed in 140310..140326 {
            let mut ec = engine(seed);
            ec.debug_grant(0, "k_danggu");
            ec.state_core_mut()
                .vm_aux
                .flags
                .insert((0, "k_danggu_roster_mask".into()), mask);
            next_phase(&mut ec);
            let c = ec.open_choice().cloned().expect("k_danggu_partner");
            let first = ec.state_core().vm_aux.flags[&(0, "k_danggu_first".into())];
            if first != 0 {
                // {3,5,9} 与 {7,8,9} 内部两两冲突，唯一不冲突候选必为 Tag(0)
                assert_eq!(c.spec.options, vec![ChoiceOption::Tag(0)]);
            }
        }
    }
    // 高望(9)保底：池为 {0,1,2,4,6,9}（与9均不冲突），只要首抽非9，4名候选中必含 Tag(9)
    let mut saw_gaowang = false;
    for seed in 140330..140345 {
        let mut eg = engine(seed);
        eg.debug_grant(0, "k_danggu");
        eg.state_core_mut().vm_aux.flags.insert(
            (0, "k_danggu_roster_mask".into()),
            (1 << 0) | (1 << 1) | (1 << 2) | (1 << 4) | (1 << 6) | (1 << 9),
        );
        next_phase(&mut eg);
        let c = eg.open_choice().cloned().expect("k_danggu_partner");
        let first = eg.state_core().vm_aux.flags[&(0, "k_danggu_first".into())];
        if first != 9 {
            assert!(c.spec.options.contains(&ChoiceOption::Tag(9)), "scs_gaowang(9) 保底纳入");
            saw_gaowang = true;
        }
    }
    assert!(saw_gaowang);

    // ---- ①③④⑤⑥ 党锢结党 -> 殁亡濒死休整 -> 准备阶段归来重结党完整链路 ----
    let mut e = engine(140350);
    e.debug_grant(0, "k_danggu");
    e.debug_grant(0, "k_mowang");
    next_phase(&mut e);
    let c_dg = e.open_choice().cloned().expect("首轮 k_danggu_partner");
    assert_eq!(c_dg.spec.prompt, "k_danggu_partner");
    assert!(c_dg.spec.options.len() >= 2);
    let _ = assert_ai_pending_choice_no_leak(&e);

    // ③ 分支对比：选伙伴 A vs 选伙伴 B 授予不同伴随技能与掩码
    let mut e_branch_b = Engine::restore_with_extensions(&e.save(), &[&epic_pack()]).unwrap();
    let tag_a = c_dg.spec.options[0];
    let tag_b = c_dg.spec.options[1];
    assert_archive_roundtrip_and_answer(&mut e, vec![tag_a]);
    let _ = submit_cmd(
        &mut e_branch_b,
        0,
        GameAction::Answer {
            choice_id: c_dg.id,
            selection: vec![tag_b],
        },
    );
    let mask_a = e.state_core().vm_aux.flags[&(0, "k_danggu_active_mask".into())];
    let mask_b = e_branch_b.state_core().vm_aux.flags[&(0, "k_danggu_active_mask".into())];
    assert_ne!(mask_a, mask_b, "选择不同侍从伙伴产生不同结党掩码");
    assert_eq!(mask_a.count_ones(), 2);
    drain(&mut e);

    // ④ 赋予手牌/装备/判定牌并置横置+翻面，制造致命伤害触发 k_mowang 濒死休整
    advance_to_play(&mut e, 0);
    let eq = e.debug_place(0, "zhuge");
    e.debug_give_hand_card(0, 0, eq);
    let _ = submit_cmd(&mut e, 0, GameAction::PlayCard { card: eq, targets: Vec::new() });
    drain(&mut e);
    let _ = e.debug_place_judge(0, "le");
    e.state_core_mut().player_mut(0).linked = true;
    e.debug_set_face_down(0, true);
    e.debug_set_hp(0, 1);
    let _ = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    drain(&mut e);

    assert_eq!(e.state_core().vm_aux.flags[&(0, "k_mowang_resting".into())], 1, "进入休整");
    assert_eq!(e.debug_hp(0), 1, "濒死休整将体力拉回 1");
    assert!(!e.debug_linked(0) && !e.debug_face_down(0), "复原横置与翻面");
    assert_eq!(e.state_core().vm_aux.flags[&(0, "k_danggu_active_mask".into())], 0, "旧党羽已注销");
    assert!(
        e.debug_hand(0).is_empty() && e.debug_equips(0).is_empty() && e.debug_judge(0).is_empty(),
        "休整弃置手牌、装备区与判定区所有牌"
    );

    // 推进到下回合 0号位准备阶段（Phase::Prepare, begin: true）：退出休整、回满体力、摸2张、重开党锢
    for _ in 0..32 {
        next_phase(&mut e);
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "k_danggu_partner" {
                break;
            }
        }
        drain(&mut e);
    }
    let c_return = e.open_choice().cloned().expect("归来重开 k_danggu_partner");
    assert_eq!(c_return.spec.prompt, "k_danggu_partner");
    assert_eq!(e.state_core().vm_aux.flags[&(0, "k_mowang_resting".into())], 0, "已退出休整");
    assert_eq!(e.debug_hp(0), e.state_core().player(0).max_hp as i32, "体力恢复至上限");
    assert_eq!(e.debug_hand(0).len(), 2, "归来摸 2 张牌");
    let _ = assert_ai_pending_choice_no_leak(&e);
    assert_archive_roundtrip_and_answer(&mut e, vec![c_return.spec.options[0]]);
    drain(&mut e);
    assert_eq!(
        e.state_core().vm_aux.flags[&(0, "k_danggu_roster_mask".into())].count_ones(),
        6,
        "两轮结党后剩余 6 名常侍"
    );

    // ① 否定条件：当 k_danggu_roster_mask == 0 时，k_mowang 不再拦截濒死
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "k_danggu_roster_mask".into()), 0);
    e.debug_set_hp(0, 1);
    let _ = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(e.debug_hp(0) <= 0, "无剩余常侍时 k_mowang 不拦截濒死");
}
