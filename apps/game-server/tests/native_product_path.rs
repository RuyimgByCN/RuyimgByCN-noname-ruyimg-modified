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
