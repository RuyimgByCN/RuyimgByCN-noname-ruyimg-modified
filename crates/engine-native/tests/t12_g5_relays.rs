//! T12 G5 跨进程接力批量切片（2026-10-14）：为已有 G1 生命周期的转换包
//! 逐包补 G5「同内容锁真双进程存档接力 + 错版/缺包拒绝」——U06 欢乐卡
//! 牌模式（g1_huanle::huanle_g5_cross_process_relay）泛化。
//!
//! 覆盖面（8 包）：ow/xianjian/hearth/gwent/yunchou/d3/wuxing/swd/mtg
//! 中的 8 个——d3 的 assets 无 cards.json（武将装载面 bridge_generals），
//! 与 ow/xianjian 同属武将包装载切片另行登记。
//! 每包：父进程建局（扩展目录合并）→ NGSA 落盘 → 子进程（本测试二进制
//! + T12_G5_WORKER 环境变量识别）同锁恢复（错版/缺包拒绝断言）→ 续推
//! 进 → 回写 → 父进程校验 revision 前进。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

static DEDUP: AtomicU64 = AtomicU64::new(9_000_000_000);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

/// 推进一步：有挂起询问则妥善应答（选牌类取前 min 个；确认类取首项），
/// 否则当前座位 NextPhase。父/子进程共用。
fn advance_once(e: &mut Engine) -> CommandOutcome {
    if let Some(c) = e.open_choice() {
        let sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(c.spec.min.max(1))
            .collect();
        let selection = if !sel.is_empty() {
            sel
        } else if !c.spec.options.is_empty() {
            vec![c.spec.options[0].clone()]
        } else {
            vec![]
        };
        let (cid, obs) = (c.id, c.observer);
        return submit(e, obs, GameAction::Answer { choice_id: cid, selection });
    }
    let seat = e.turn().current;
    submit(e, seat, GameAction::NextPhase)
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 从 drop-in 根桥接包（导入记录的 extension_root 指回 drop-in 目录）。
fn pack_from_repo(dir: &Path, package_id: &str, pack_name: &'static str) -> ExtensionPack {
    let store = PackageStore::open(dir.join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options())
        .expect("import（幂等）");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == package_id)
        .unwrap_or_else(|| panic!("{package_id} 安装"))
        .clone();
    let json = std::fs::read_to_string(
        Path::new(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap_or_else(|e| panic!("读 cards.json: {e}"));
    let out = content_native_loader::bridge_pack(&json).expect("桥接");
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    out.pack
}

/// 共享接力：父进程建局+落盘，子进程同锁恢复（错版/缺包拒绝）+推进+回写。
fn g5_relay(package_id: &str, pack_name: &'static str, test_name: &str) {
    use engine_native::archive::{decode_archive, encode_archive};

    // 子进程分支：读档→错版/缺包拒绝→同锁恢复→推进→回写
    if let Ok(archive_path) = std::env::var("T12_G5_ARCHIVE") {
        // 子进程 dedup 计数器与父进程独立——高段偏移避开存档内已用键
        //（N10：重复键按重复命令干净拒绝）。
        DEDUP.fetch_add(500_000_000, Ordering::Relaxed);
        let reply_path = std::env::var("T12_G5_REPLY").expect("reply path");
        let tmp = std::env::var("T12_G5_STORE").expect("store dir");
        let pack = pack_from_repo(Path::new(&tmp), package_id, pack_name);
        let bytes = std::fs::read(&archive_path).expect("子进程读存档");
        let snap = decode_archive(&bytes).expect("子进程解码");
        // 错版拒绝
        let mut wrong = pack_from_repo(Path::new(&tmp), package_id, pack_name);
        wrong.def.version += 1;
        assert!(
            Engine::restore_with_extensions(&snap, &[&wrong]).is_err(),
            "错版拒绝"
        );
        // 缺包拒绝
        assert!(Engine::restore(&snap).is_err(), "缺扩展包拒绝");
        // 同锁恢复 + 推进
        let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("子进程恢复");
        let rev0 = e.revision();
        for _ in 0..8 {
            let out = advance_once(&mut e);
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "子进程推进被拒: {out:?}"
            );
        }
        assert!(e.revision() > rev0, "子进程 revision 前进");
        std::fs::write(&reply_path, encode_archive(&e.save())).expect("子进程回写");
        let _ = test_name;
        return;
    }

    // 父进程：建局（扩展目录合并）→ 推进数步 → 落盘 → 自启子进程 → 校验
    let dir = tempfile::tempdir().expect("临时目录");
    let store_dir = dir.path().join("store");
    let pack = pack_from_repo(dir.path(), package_id, pack_name);
    let mut e =
        Engine::new_with_extensions(7000_000 + test_name.len() as u64, 3, &[&pack])
            .expect("建局");
    let base = e.revision();
    for _ in 0..6 {
        advance_once(&mut e);
    }
    assert!(e.revision() >= base, "父进程推进");
    let archive_path = dir.path().join("relay.ngsa");
    let reply_path = dir.path().join("relay.result.ngsa");
    std::fs::write(&archive_path, encode_archive(&e.save())).expect("父进程落盘");

    let exe = std::env::current_exe().expect("测试二进制");
    let status = std::process::Command::new(exe)
        .args(["--exact", test_name, "--nocapture"])
        .env("T12_G5_ARCHIVE", archive_path.display().to_string())
        .env("T12_G5_REPLY", reply_path.display().to_string())
        .env("T12_G5_STORE", store_dir.display().to_string())
        .output()
        .expect("自启子进程");
    assert!(
        status.status.success(),
        "子进程接力失败: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let reply = decode_archive(
        &std::fs::read(&reply_path).expect("读回写存档"),
    )
    .expect("解码回写");
    let r = Engine::restore_with_extensions(&reply, &[&pack]).expect("父进程恢复回写");
    assert!(r.revision() > base, "跨进程接力：子进程推进在回写存档中");
}

// 九包 G5 接力（G1 生命周期已验收的转换包；huanle 已单独完成）。

/// d3 专用：character.json 桥（pack + generals 双产物）。
fn d3_pack_from_repo(dir: &Path) -> ExtensionPack {
    let store = PackageStore::open(dir.join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options())
        .expect("import（幂等）");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "d3-select")
        .expect("d3-select 安装")
        .clone();
    let json = std::fs::read_to_string(
        Path::new(meta.extension_root.as_ref().unwrap()).join("assets/character.json"),
    )
    .expect("读 character.json");
    let out = content_native_loader::bridge_pack(&json).expect("桥接");
    let _generals = content_native_loader::bridge_generals(&json).expect("generals");
    out.pack
}

#[test]
fn t12_g5_relay_shaai_ow() {
    g5_relay("shaai-ow", "shaai-ow-upstream-noname", "t12_g5_relay_shaai_ow");
}

#[test]
fn t12_g5_relay_shaai_xianjian() {
    g5_relay("shaai-xianjian", "shaai-xianjian-upstream-noname", "t12_g5_relay_shaai_xianjian");
}

#[test]
fn t12_g5_relay_shahai_hearth() {
    g5_relay("shahai-hearth", "shahai-hearth-upstream-noname", "t12_g5_relay_shahai_hearth");
}

#[test]
fn t12_g5_relay_shahai_gwent() {
    g5_relay("shahai-gwent", "shahai-gwent-upstream-noname", "t12_g5_relay_shahai_gwent");
}

#[test]
fn t12_g5_relay_shahai_yunchou() {
    g5_relay("shahai-yunchou", "shahai-yunchou-upstream-noname", "t12_g5_relay_shahai_yunchou");
}

#[test]
fn t12_g5_relay_wuxing() {
    g5_relay("wuxing", "wuxing-upstream-liuli", "t12_g5_relay_wuxing");
}

#[test]
fn t12_g5_relay_shahai_swd() {
    g5_relay("shahai-swd", "shahai-swd-upstream-noname", "t12_g5_relay_shahai_swd");
}

#[test]
fn t12_g5_relay_shahai_mtg() {
    g5_relay("shahai-mtg", "shahai-mtg-upstream-noname", "t12_g5_relay_shahai_mtg");
}

// ---- 第三批：gujian/wudian/zhenfa（cards.json 桥）+ cardpile（pile 桥）----
// + d3（character.json 桥，pack+generals）。

#[test]
fn t12_g5_relay_shahai_gujian() {
    g5_relay(
        "shahai-gujian",
        "shahai-gujian-upstream-noname",
        "t12_g5_relay_shahai_gujian",
    );
}

#[test]
fn t12_g5_relay_wudian() {
    g5_relay("wudian", "wudian-upstream-noname", "t12_g5_relay_wudian");
}

#[test]
fn t12_g5_relay_shahai_zhenfa() {
    g5_relay(
        "shahai-zhenfa",
        "shahai-zhenfa-upstream-noname",
        "t12_g5_relay_shahai_zhenfa",
    );
}

#[test]
fn t12_g5_relay_d3_generals() {
    let dir = tempfile::tempdir().expect("临时目录");
    let store_dir = dir.path().join("store");
    let pack = d3_pack_from_repo(dir.path());
    let mut e =
        Engine::new_with_extensions(7000_999, 3, &[&pack]).expect("建局");
    let base = e.revision();
    for _ in 0..6 {
        advance_once(&mut e);
    }
    assert!(e.revision() >= base, "父进程推进");
    let archive_path = dir.path().join("relay.ngsa");
    let reply_path = dir.path().join("relay.result.ngsa");
    let exe = std::env::current_exe().expect("测试二进制");
    if std::env::var("T12_G5_ARCHIVE").is_ok() {
        return; // 本测试无子进程分支（父进程模式直测）
    }
    std::fs::write(&archive_path, engine_native::archive::encode_archive(&e.save()))
        .expect("落盘");
    let status = std::process::Command::new(&exe)
        .args(["--exact", "t12_g5_relay_d3_generals_worker", "--nocapture"])
        .env("T12_G5_ARCHIVE", archive_path.display().to_string())
        .env("T12_G5_REPLY", reply_path.display().to_string())
        .env("T12_G5_STORE", store_dir.display().to_string())
        .output()
        .expect("自启子进程");
    assert!(status.status.success(), "子进程失败");
    let r = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(
            &std::fs::read(&reply_path).expect("读回写"),
        )
        .expect("解码"),
        &[&pack],
    )
    .expect("恢复回写");
    assert!(r.revision() > base, "跨进程接力 revision 前进");
}

/// d3 跨进程 worker（bridge_generals 面）。
#[test]
fn t12_g5_relay_d3_generals_worker() {
    if std::env::var("T12_G5_ARCHIVE").is_err() {
        return; // 仅 worker 模式运行
    }
    let archive_path = std::env::var("T12_G5_ARCHIVE").unwrap();
    let reply_path = std::env::var("T12_G5_REPLY").unwrap();
    let store_dir = std::env::var("T12_G5_STORE").unwrap();
    DEDUP.fetch_add(500_000_000, Ordering::Relaxed);
    let pack = d3_pack_from_repo(Path::new(&store_dir).parent().expect("父目录"));
    let snap = engine_native::archive::decode_archive(
        &std::fs::read(&archive_path).expect("读档"),
    )
    .expect("解码");
    let mut wrong = d3_pack_from_repo(Path::new(&store_dir).parent().expect("父目录"));
    wrong.def.version += 1;
    assert!(Engine::restore_with_extensions(&snap, &[&wrong]).is_err(), "错版拒绝");
    assert!(Engine::restore(&snap).is_err(), "缺包拒绝");
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    for _ in 0..8 {
        advance_once(&mut e);
    }
    std::fs::write(
        &reply_path,
        engine_native::archive::encode_archive(&e.save()),
    )
    .expect("回写");
}

// ---- 第四批：cardpile liuli 仓跨进程接力（双 namespace 切片收尾）----

/// cardpile liuli 仓专用：pile.json 桥（def 名 cardpile-upstream-liuli）。
fn cardpile_liuli_pack_from_repo(dir: &Path) -> ExtensionPack {
    use engine_native::content::pile::extension_pack_from_pile;
    let store = PackageStore::open(dir.join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options())
        .expect("import（幂等）");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "cardpile" && m.namespace == "upstream-liuli")
        .expect("liuli 仓 cardpile 安装")
        .clone();
    let pile_bytes = std::fs::read_to_string(
        Path::new(meta.extension_root.as_ref().unwrap()).join("assets/pile.json"),
    )
    .expect("读 pile.json");
    let spec = content_native_loader::bridge_pile(&pile_bytes).expect("pile 可装载");
    assert_eq!(spec.total, 160);
    let def = engine_native::content::extension::ExtensionDef {
        name: "cardpile-upstream-liuli",
        version: 1,
        description: "T12 G5 接力：cardpile 牌堆比例（liuli）".into(),
    };
    extension_pack_from_pile(def, "cpl", &spec).expect("桥接")
}

/// 双 namespace 切片：liuli 仓 cardpile 同内容锁跨进程接力（G5）。
#[test]
fn t12_g5_relay_cardpile_liuli() {
    use engine_native::archive::{decode_archive, encode_archive};

    if let Ok(archive_path) = std::env::var("T12_G5_ARCHIVE") {
        let reply_path = std::env::var("T12_G5_REPLY").expect("reply path");
        let store_dir = std::env::var("T12_G5_STORE").expect("store dir");
        DEDUP.fetch_add(500_000_000, Ordering::Relaxed);
        let pack = cardpile_liuli_pack_from_repo(Path::new(&store_dir));
        let snap = decode_archive(&std::fs::read(&archive_path).expect("读档")).expect("解码");
        let mut wrong = cardpile_liuli_pack_from_repo(Path::new(&store_dir));
        wrong.def.version += 1;
        assert!(Engine::restore_with_extensions(&snap, &[&wrong]).is_err(), "错版拒绝");
        assert!(Engine::restore(&snap).is_err(), "缺包拒绝");
        let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("子进程恢复");
        for _ in 0..8 {
            advance_once(&mut e);
        }
        std::fs::write(&reply_path, encode_archive(&e.save())).expect("子进程回写");
        return;
    }

    let dir = tempfile::tempdir().expect("临时目录");
    let store_dir = dir.path().join("store");
    let pack = cardpile_liuli_pack_from_repo(dir.path());
    let mut e = Engine::new_with_extensions(7001_001, 3, &[&pack]).expect("建局");
    let base = e.revision();
    for _ in 0..6 {
        advance_once(&mut e);
    }
    assert!(e.revision() >= base, "父进程推进");
    let archive_path = dir.path().join("relay.ngsa");
    let reply_path = dir.path().join("relay.result.ngsa");
    std::fs::write(&archive_path, encode_archive(&e.save())).expect("父进程落盘");
    let exe = std::env::current_exe().expect("测试二进制");
    let status = std::process::Command::new(exe)
        .args(["--exact", "t12_g5_relay_cardpile_liuli", "--nocapture"])
        .env("T12_G5_ARCHIVE", archive_path.display().to_string())
        .env("T12_G5_REPLY", reply_path.display().to_string())
        .env("T12_G5_STORE", store_dir.display().to_string())
        .output()
        .expect("自启子进程");
    assert!(
        status.status.success(),
        "子进程接力失败: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let r = Engine::restore_with_extensions(
        &decode_archive(&std::fs::read(&reply_path).expect("读回写")).expect("解码"),
        &[&pack],
    )
    .expect("父进程恢复回写");
    assert!(r.revision() > base, "跨进程接力 revision 前进");
}
