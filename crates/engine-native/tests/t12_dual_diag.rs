//! 双仓同载挂起诊断（2026-10-14）：分阶段计时定位挂起点。

use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::ExtensionPack;
use engine_native::Engine;
use std::path::{Path, PathBuf};
use std::time::Instant;

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

fn cardpile_pack_from_repo(
    dir: &Path,
    namespace: &'static str,
    def_name: &'static str,
    prefix: &'static str,
) -> ExtensionPack {
    use engine_native::content::pile::extension_pack_from_pile;
    let store = PackageStore::open(dir.join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options())
        .expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "cardpile" && m.namespace == namespace)
        .unwrap_or_else(|| panic!("{namespace}/cardpile 安装"))
        .clone();
    let pile = std::fs::read_to_string(
        Path::new(meta.extension_root.as_ref().unwrap()).join("assets/pile.json"),
    )
    .expect("读 pile.json");
    let spec = content_native_loader::bridge_pile(&pile).expect("pile 可装载");
    let def = engine_native::content::extension::ExtensionDef {
        name: def_name,
        version: 1,
        description: "诊断".into(),
    };
    extension_pack_from_pile(def, prefix, &spec).expect("桥接")
}

#[test]
fn diag_dual_namespace_hang_point() {
    let dir = tempfile::tempdir().unwrap();
    println!("[diag] 开始双仓桥接");
    let pack_a = cardpile_pack_from_repo(dir.path(), "upstream-noname", "cardpile-upstream-noname", "cnp");
    let pack_b = cardpile_pack_from_repo(dir.path(), "upstream-liuli", "cardpile-upstream-liuli", "cpl");
    println!("[diag] 桥接完成: noname={}张 liuli={}张", pack_a.card_count(), pack_b.card_count());

    println!("[diag] 双仓建局开始");
    let t0 = Instant::now();
    let mut e = Engine::new_with_extensions(7100_001, 3, &[&pack_a, &pack_b]).expect("建局");
    println!("[diag] 双仓建局: {:?} cards={}", t0.elapsed(), e.catalog().cards.len());

    for i in 0..30 {
        let ts = Instant::now();
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        let out = e.submit(engine_native::GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: 7000_0000 + i as u64,
            action: engine_native::GameAction::NextPhase,
        });
        println!(
            "[diag] step {i}: {:?} seat={seat} elapsed={:?} hand_counts={:?}",
            out,
            ts.elapsed(),
            (0..3u8).map(|s| e.debug_hand(s).len()).collect::<Vec<_>>()
        );
        if e.outcome().is_some() {
            println!("[diag] 终局于 step {i}");
            break;
        }
        if ts.elapsed() > Duration::from_secs(30) {
            println!("[diag] 单步 >30s——确认挂起于 step {i}");
            break;
        }
    }
    println!("[diag] 完成");
}

use std::time::Duration;
