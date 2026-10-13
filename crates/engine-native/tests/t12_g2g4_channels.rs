//! T12 G2–G4 产品通道验收·五断言模板批量复制（2026-10-14）。
//!
//! 以 U06 欢乐卡牌（g1_huanle::u06_g2g4_channels_acceptance）的五断言为
//! 模板，为其余 G1 转换包逐包复制：资源/目录搜索/界面·AI 隔离/配置。
//! 按桥型三选装配：cards.json（bridge_pack，含武将段）/ character.json
//! （bridge_pack+bridge_generals 双产物）/ pile.json（bridge_pile+
//! extension_pack_from_pile，双仓 namespace）。
//!
//! 搜索探针分两类：Card(目录卡名片段) 与 General(武将名片段)。

use std::path::{Path, PathBuf};

use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::ExtensionPack;
use engine_native::view::HandView;
use engine_native::Engine;

const SEED: u64 = 7500;

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

struct Loaded {
    package_id: String,
    namespace: &'static str,
    pack: ExtensionPack,
    fingerprint_bytes: Vec<u8>,
    probe: Probe,
}

#[derive(Debug, Clone, Copy)]
enum Probe {
    Card(&'static str),
    General(&'static str),
}

fn installed_root(dir: &Path, package_id: &str, namespace: &str) -> PathBuf {
    let store = PackageStore::open(dir.join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options())
        .expect("import（幂等）");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == package_id && m.namespace == namespace)
        .unwrap_or_else(|| panic!("{namespace}/{package_id} 安装"))
        .clone();
    PathBuf::from(meta.extension_root.as_ref().expect("extension_root"))
}

fn load_cards(
    dir: &Path,
    package_id: &str,
    namespace: &'static str,
    pack_name: &'static str,
    probe: Probe,
) -> Loaded {
    let root = installed_root(dir, package_id, namespace);
    let npack = std::fs::read(root.join("package.npack.json")).expect("读 npack");
    let cards = std::fs::read_to_string(root.join("assets/cards.json")).expect("读 cards");
    let out = content_native_loader::bridge_pack(&cards).expect("桥接");
    assert_eq!(out.pack.def.name, pack_name);
    Loaded {
        package_id: package_id.to_string(),
        namespace,
        pack: out.pack,
        fingerprint_bytes: npack,
        probe,
    }
}

fn load_character(
    dir: &Path,
    package_id: &str,
    namespace: &'static str,
    pack_name: &'static str,
    probe: Probe,
) -> Loaded {
    let root = installed_root(dir, package_id, namespace);
    let npack = std::fs::read(root.join("package.npack.json")).expect("读 npack");
    let json = std::fs::read_to_string(root.join("assets/character.json")).expect("读 character");
    let out = content_native_loader::bridge_pack(&json).expect("桥接");
    let _generals = content_native_loader::bridge_generals(&json).expect("generals");
    assert_eq!(out.pack.def.name, pack_name);
    Loaded {
        package_id: package_id.to_string(),
        namespace,
        pack: out.pack,
        fingerprint_bytes: npack,
        probe,
    }
}

fn load_pile(
    dir: &Path,
    namespace: &'static str,
    package_id: &str,
    def_name: &'static str,
) -> Loaded {
    use engine_native::content::pile::extension_pack_from_pile;
    let root = installed_root(dir, package_id, namespace);
    let pile = std::fs::read_to_string(root.join("assets/pile.json")).expect("读 pile");
    let spec = content_native_loader::bridge_pile(&pile).expect("pile 可装载");
    let def = engine_native::content::extension::ExtensionDef {
        name: def_name,
        version: 1,
        description: "T12 G2–G4 通道验收".into(),
    };
    let prefix = if namespace == "upstream-liuli" { "cpl" } else { "cnp" };
    let pack = extension_pack_from_pile(def, prefix, &spec).expect("桥接");
    let bytes = std::fs::read(root.join("package.npack.json")).expect("npack bytes");
    Loaded {
        package_id: package_id.to_string(),
        namespace,
        pack,
        fingerprint_bytes: bytes,
        probe: Probe::Card(if namespace == "upstream-liuli" {
            "cpl_sha"
        } else {
            "cnp_sha"
        }),
    }
}

/// 五断言主体：资源/目录搜索/界面·AI 隔离/配置。
fn assert_channels(
    dir: &Path,
    loaded: &Loaded,
    seed: u64,
    expected_cards: Option<usize>,
    general_probe: Option<&'static str>,
) {
    // —— 资源（G4）：指纹文件字节 content_hash 与安装记录一致 ——
    let content_hash = content_runtime::content_hash_hex(&loaded.fingerprint_bytes);
    {
        let store = PackageStore::open(dir.join("store")).unwrap();
        let report = import_extensions(&store, &repo_extensions_root(), &options())
            .expect("import");
        let meta = report
            .installed
            .iter()
            .find(|m| {
                m.package_id == loaded.package_id && m.namespace == loaded.namespace
            })
            .unwrap_or_else(|| panic!("{}/{} 安装记录", loaded.namespace, loaded.package_id));
        assert_eq!(
            meta.content_hash, content_hash,
            "资源指纹保真（{}）",
            loaded.namespace
        );
    }

    // —— 目录+搜索（G2/G3）——
    let mut e = Engine::new_with_extensions(seed, 3, &[&loaded.pack]).expect("建局");
    if let Some(n) = expected_cards {
        let base = Engine::new(1, 3).catalog().cards.len();
        assert_eq!(e.catalog().cards.len(), base + n, "扩展卡 {n} 张入目录");
    }
    let frag = general_probe.unwrap_or(match loaded.probe {
        Probe::Card(f) => f,
        Probe::General(f) => f,
    });
    let hit = {
        let cat = e.catalog();
        cat.cards
            .iter()
            .filter(|c| c.name.contains(frag))
            .count()
            + (0..cat.general_count() as u16)
                .filter_map(|i| cat.general_meta(i))
                .filter(|(name, ..)| name.contains(frag))
                .count()
    };
    assert!(hit > 0, "搜索：探针 {frag} 命中");

    // —— 界面/AI 隔离（G4）：一张标准卡入手牌核验 Own/Hidden ——
    let probe_card = {
        let cat = e.catalog();
        cat.cards
            .iter()
            .position(|c| c.name == "sha")
            .expect("标准杀在目录") as u32
    };
    {
        let core = e.state_core_mut();
        core.players[1].hand.push(probe_card);
    }
    let v1 = e.ai_session(1).view();
    match (&v1.players[1].hand, &v1.players[2].hand) {
        (HandView::Own(mine), other) => {
            assert!(mine.contains(&probe_card), "持有者视角可见手牌");
            assert!(!matches!(other, HandView::Own(_)), "他人手牌不泄露");
        }
        _ => panic!("持有者手牌应为 Own 明细"),
    }
    let v2 = e.ai_session(2).view();
    assert!(
        matches!(v2.players[1].hand, HandView::Hidden),
        "他人视角手牌 Hidden"
    );
    drop(e);

    // —— 配置（G3）：store 重开激活集存续 ——
    let store = PackageStore::open(dir.join("store")).unwrap();
    let active = store.read_active().unwrap();
    let slot_key = format!("{}/{}", loaded.namespace, loaded.package_id);
    assert!(
        active.packages.contains_key(&slot_key),
        "store 激活集持久（{slot_key}）"
    );
}

macro_rules! cards_case {
    ($fname:ident, $pid:expr, $ns:expr, $pname:expr, $probe:expr, $n:expr, $seed:expr) => {
        #[test]
        fn $fname() {
            let dir = tempfile::tempdir().unwrap();
            let l = load_cards(dir.path(), $pid, $ns, $pname, $probe);
            let base = Engine::new(1, 3).catalog().cards.len();
            let mut e = Engine::new_with_extensions(SEED + $seed, 3, &[&l.pack])
                .expect("建局");
            assert_eq!(
                e.catalog().cards.len(),
                base + $n,
                "扩展卡 {} 张入目录",
                $n
            );
            let _ = &mut e;
            assert_channels(dir.path(), &l, SEED + $seed, None, None);
        }
    };
}

cards_case!(g2g4_shahai_hearth, "shahai-hearth", "upstream-noname", "shahai-hearth-upstream-noname", Probe::Card("ht_linghunzhihuo"), 17, 2);
cards_case!(g2g4_shahai_gwent, "shahai-gwent", "upstream-noname", "shahai-gwent-upstream-noname", Probe::Card("gw_yanziyaoshui"), 21, 3);
cards_case!(g2g4_shahai_yunchou, "shahai-yunchou", "upstream-noname", "shahai-yunchou-upstream-noname", Probe::Card("yc_huoshan"), 23, 4);
cards_case!(g2g4_shahai_mtg, "shahai-mtg", "upstream-noname", "shahai-mtg-upstream-noname", Probe::Card("mtg_longlushanfeng"), 10, 6);
cards_case!(g2g4_shahai_gujian, "shahai-gujian", "upstream-noname", "shahai-gujian-upstream-noname", Probe::Card("gj_shujinsan"), 40, 8);
cards_case!(g2g4_shahai_zhenfa, "shahai-zhenfa", "upstream-noname", "shahai-zhenfa-upstream-noname", Probe::Card("zf_dizaizhen"), 8, 10);

#[test]
fn g2g4_shahai_swd() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_cards(
        dir.path(),
        "shahai-swd",
        "upstream-noname",
        "shahai-swd-upstream-noname",
        Probe::Card("swd_hufu"),
    );
    assert_channels(dir.path(), &l, SEED + 5, None, None);
}

#[test]
fn g2g4_wuxing() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_cards(
        dir.path(),
        "wuxing",
        "upstream-liuli",
        "wuxing-upstream-liuli",
        Probe::Card("wxl_wuxingpan"),
    );
    assert_channels(dir.path(), &l, SEED + 7, None, None);
}

#[test]
fn g2g4_shaai_ow() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_cards(
        dir.path(),
        "shaai-ow",
        "upstream-noname",
        "shaai-ow-upstream-noname",
        Probe::General("士兵76"),
    );
    let base_g = Engine::new(1, 3).catalog().general_count();
    let mut e = Engine::new_with_extensions(SEED, 3, &[&l.pack]).expect("建局");
    assert_eq!(e.catalog().general_count(), base_g + 24, "ow 24 将入目录");
    let _ = &mut e;
    assert_channels(
                dir.path(),
                &l,
                SEED,
        None,
        Some("士兵76"),
    );
}

#[test]
fn g2g4_shaai_xianjian() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_cards(
        dir.path(),
        "shaai-xianjian",
        "upstream-noname",
        "shaai-xianjian-upstream-noname",
        Probe::General("赵灵儿"),
    );
    let base_g = Engine::new(1, 3).catalog().general_count();
    let mut e = Engine::new_with_extensions(SEED + 1, 3, &[&l.pack]).expect("建局");
    assert_eq!(e.catalog().general_count(), base_g + 36, "xianjian 36 将入目录");
    let _ = &mut e;
    assert_channels(
                dir.path(),
                &l,
                SEED + 1,
        None,
        Some("赵灵儿"),
    );
}

#[test]
fn g2g4_wudian() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_cards(
        dir.path(),
        "wudian",
        "upstream-noname",
        "wudian-upstream-noname",
        Probe::Card("wd_"),
    );
    assert_channels(dir.path(), &l, SEED + 9, None, None);
}

#[test]
fn g2g4_d3_select() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_character(
        dir.path(),
        "d3-select",
        "upstream-noname",
        "d3-upstream-noname",
        Probe::General("ddd刘冲"),
    );
    let base_g = Engine::new(1, 3).catalog().general_count();
    let mut e = Engine::new_with_extensions(SEED + 13, 3, &[&l.pack]).expect("建局");
    assert_eq!(e.catalog().general_count(), base_g + 1, "d3 1 将入目录");
    let _ = &mut e;
    assert_channels(
                dir.path(),
                &l,
                SEED + 13,
        None,
        Some("ddd刘冲"),
    );
}

#[test]
fn g2g4_cardpile_noname() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_pile(
        dir.path(),
        "upstream-noname",
        "cardpile",
        "cardpile-upstream-noname",
    );
    let base = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(SEED + 11, 3, &[&l.pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base + 113, "noname 仓 113 张");
    let _ = &mut e;
    assert_channels(dir.path(), &l, SEED + 11, None, None);
}

#[test]
fn g2g4_cardpile_liuli() {
    let dir = tempfile::tempdir().unwrap();
    let l = load_pile(
        dir.path(),
        "upstream-liuli",
        "cardpile",
        "cardpile-upstream-liuli",
    );
    let base = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(SEED + 12, 3, &[&l.pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base + 113, "liuli 仓 113 张");
    let _ = &mut e;
    assert_channels(dir.path(), &l, SEED + 12, None, None);
}
