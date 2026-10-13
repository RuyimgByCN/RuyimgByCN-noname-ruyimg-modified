//! T6③ 产品路径回归：安装包 → 装载 → 原生建局 → 有界 AI 驱动。
//!
//! 全链路在服务端同进程内验证（HTTP 层薄封装，逻辑在
//! `src/native.rs`）：content-runtime 安装 → content-native-loader
//! 桥接 → engine-native 建局 → BoundedPlanner 驱动（T13 衔接）。

#[path = "../src/native.rs"]
mod native;

use native::{create_native_game, NativeContent, NativeGameRequest};
use std::path::PathBuf;

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// 全链路：安装欢乐卡牌 → 装载 → 带扩展建局 → AI 驱动推进。
#[test]
fn t63_import_load_build_and_drive_with_extension() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    let installed = content.import().expect("安装包");
    assert!(
        installed.iter().any(|p| p.package_id == "huanle-cards"),
        "欢乐卡牌应安装"
    );
    let listed = content.installed().expect("列出已安装");
    assert!(
        listed.iter().any(|p| p.package_id == "huanle-cards"),
        "存储应能列出已安装包"
    );

    let loaded = content.load("huanle-cards").expect("装载");
    assert_eq!(loaded.pack.card_count(), 7, "与 G1 装载口径一致（含伴随技）");

    let req = NativeGameRequest {
        packages: vec!["huanle-cards".into()],
        seed: 7301,
        seats: 3,
        max_steps: 600,
        planner_nodes: 64,
    };
    let summary = create_native_game(&content, &req).expect("建局+驱动");
    assert_eq!(
        summary.catalog_cards, summary.base_catalog_cards + 7,
        "扩展牌应并入目录"
    );
    assert!(summary.steps > 0, "驱动器应有推进");
    assert!(
        !summary.stalled,
        "驱动器不得停滞（steps={} finished={}）",
        summary.steps, summary.finished
    );
    println!(
        "T6③ 带扩展实测: steps={} finished={} winner={:?} elapsed_ms={}",
        summary.steps, summary.finished, summary.winner, summary.elapsed_ms
    );
}

/// T8 增量 4：drop-in 编译包（package.npack.json 含 rule-vm 模块）→
/// 安装 → 装载器 bridge_vm_skill_decls → 建局 → 触发落账——不重编译
/// 安装含新技能的包（C1 数据流的服务端磁盘路径）。
#[test]
fn t8_native_disk_path_vm_package_end_to_end() {
    const JIUWEI_DSL: &str = "\
skill hnl_jiuwei_t8(990001) {
  trigger phase_end owner_is_target {
    if hp(self) < max_hp(self) {
      effect recover_hp(self, 1);
    } else {
      effect draw_cards(self, 1);
    }
  }
}
";
    let pkg = content_compiler::compile_source_named("t8-disk-vm", JIUWEI_DSL)
        .expect("DSL 编译");
    let npack_json = serde_json::to_string(&pkg).expect("npack 序列化");

    // drop-in 目录：仅 package.npack.json（vm-only 包，无 cards.json）
    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("t8-vm-pack");
    std::fs::create_dir_all(&dropin).unwrap();
    std::fs::write(dropin.join("package.npack.json"), npack_json).unwrap();

    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store"))
            .expect("open");
    let installed = content.import().expect("安装编译包");
    assert!(
        installed.iter().any(|p| p.package_id == "t8-disk-vm"),
        "编译包应安装"
    );

    let loaded = content.load("t8-disk-vm").expect("装载");
    assert_eq!(loaded.pack.vm_skills.len(), 1, "VM 声明经磁盘装载入包");
    assert_eq!(loaded.pack.vm_skills[0].name, "hnl_jiuwei_t8_phase_end");

    // 建局 → 授技 → 推进到持有者阶段结束：VM 效果落账（hp 2→3）
    let mut e =
        engine_native::Engine::new_with_extensions(920001, 4, &[&loaded.pack])
            .expect("建局");
    e.debug_grant(1, "hnl_jiuwei_t8_phase_end");
    e.debug_set_hp(1, 2);
    let mut converged = false;
    for _ in 0..40 {
        if e.turn().current == 1 && e.debug_hp(1) == 3 {
            converged = true;
            break;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        let actor = e.turn().current;
        submit_next_phase(&mut e, actor);
        answer_pending(&mut e);
    }
    assert!(converged, "VM 技能：服务端磁盘路径阶段结束受伤回 1");
}

fn submit_next_phase(e: &mut engine_native::Engine, actor: u8) {
    let rev = e.revision();
    let _ = e.submit(engine_native::GameCommand {
        actor,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action: engine_native::GameAction::NextPhase,
    });
}

fn answer_pending(e: &mut engine_native::Engine) {
    if let Some(c) = e.open_choice() {
        let sel: Vec<engine_native::ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, engine_native::ChoiceOption::Card(_)))
            .copied()
            .take(c.spec.min.max(1))
            .collect();
        let (cid, obs) = (c.id, c.observer);
        let selection = if sel.is_empty() && !c.spec.options.is_empty() {
            vec![c.spec.options[0].clone()]
        } else {
            sel
        };
        let rev = e.revision();
        let _ = e.submit(engine_native::GameCommand {
            actor: obs,
            expected_revision: rev,
            dedup_id: (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64)
                | 1,
            action: engine_native::GameAction::Answer {
                choice_id: cid,
                selection,
            },
        });
    }
}

/// 纯标准目录建局 + 驱动（无扩展包的产品路径基线）。
#[test]
fn t63_standard_only_build_and_drive() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    let req = NativeGameRequest {
        packages: vec![],
        seed: 7302,
        seats: 3,
        max_steps: 300,
        planner_nodes: 64,
    };
    let summary = create_native_game(&content, &req).expect("建局+驱动");
    assert_eq!(
        summary.catalog_cards, summary.base_catalog_cards,
        "无扩展时目录等于标准基线"
    );
    assert!(summary.steps > 0);
    assert!(!summary.stalled, "标准对局驱动器不得停滞");
    println!(
        "T6③ 标准实测: steps={} finished={} elapsed_ms={}",
        summary.steps, summary.finished, summary.elapsed_ms
    );
}

/// 未安装包装载必须可处理报错（不 panic）。
#[test]
fn t63_load_unknown_package_errors_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    let err = match content.load("no-such-package") {
        Err(err) => err,
        Ok(_) => panic!("未安装包装载应报错"),
    };
    assert!(err.contains("包未安装"), "错误应指明未安装: {err}");
}
