//! NC08：「组合」的**技能/武将层面**矩阵（承接台账第 178 行的边界结论）。
//!
//! 第 178 行指出：卡牌配对矩阵的「目录 = 两包之和 + 标准牌」不变式**不适用于**
//! 纯武将/纯技能包与牌堆补丁包 ⇒ 若要覆盖它们，**须另立判据**。本文件即该判据：
//!
//! 对**每个含 `cards.json` 或 `character.json` 的包**，记 `(cards, skills, generals)`；两包同载后断言：
//! 1. `cards = a.cards + b.cards + 标准牌数`；
//! 2. `skills = a.skills + b.skills + 标准技能数`；
//! 3. `generals = a.generals + b.generals`。
//!
//! **不做的事**：不把 `pile.json` 纳入（替换牌堆语义，卡片求和无意义）。

use std::path::PathBuf;

use engine_native::content::extension::ExtensionPack;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn load(pkg_dir: &PathBuf) -> Option<(String, ExtensionPack)> {
    let assets = pkg_dir.join("assets");
    let mut json = String::new();
    for f in ["cards.json", "character.json"] {
        let p = assets.join(f);
        if p.is_file() {
            if let Ok(t) = std::fs::read_to_string(&p) {
                json = t;
                break;
            }
        }
    }
    if json.is_empty() {
        return None;
    }
    let mut out = content_native_loader::bridge_pack(&json).ok()?;
    if let Ok(chars) = std::fs::read_to_string(assets.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            out.pack.generals = b.generals;
        }
    }
    let name = pkg_dir.file_name()?.to_string_lossy().to_string();
    Some((name, out.pack))
}

fn shape(packs: &[&ExtensionPack]) -> (usize, usize, usize) {
    let e = engine_native::Engine::new_with_extensions(67, 4, packs).expect("建局");
    (
        e.catalog().cards.len(),
        e.catalog().skills.len(),
        e.catalog().extension_generals.len(),
    )
}

#[test]
fn skill_pair_matrix_over_general_packs() {
    let root = root();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("read packages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    let std = shape(&[]);
    let mut packs: Vec<(String, ExtensionPack, (usize, usize, usize))> = Vec::new();
    for d in &dirs {
        if let Some((name, pack)) = load(d) {
            let s = shape(&[&pack]);
            // 只收「相对标准有明显增量」的包（纯空/无增量包配对无信息量）
            if s != std {
                packs.push((name, pack, s));
            }
        }
    }
    eprintln!(
        "DBG 技能矩阵：标准 (cards,skills,generals)={std:?}；参与配对 {} 个包",
        packs.len()
    );

    let mut pairs = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for i in 0..packs.len() {
        for j in (i + 1)..packs.len() {
            let (na, pa, sa) = &packs[i];
            let (nb, pb, sb) = &packs[j];
            let combined = shape(&[pa, pb]);
            pairs += 1;
            let want = (
                (sa.0 - std.0) + (sb.0 - std.0) + std.0,
                (sa.1 - std.1) + (sb.1 - std.1) + std.1,
                (sa.2 - std.2) + (sb.2 - std.2) + std.2,
            );
            if combined != want {
                failures.push(format!(
                    "{na} + {nb}: 实得 {combined:?}，期望 {want:?}（各自 {sa:?} / {sb:?}）"
                ));
            }
        }
    }
    eprintln!(
        "DBG 技能矩阵：配对 {pairs} 对、失败 {} 项",
        failures.len()
    );
    for f in failures.iter().take(10) {
        eprintln!("DBG 技能矩阵失败：{f}");
    }
    assert!(pairs > 0, "应至少有一对可配对");
    assert!(
        failures.is_empty(),
        "技能/武将配对应满足加和不变式，实得 {} 项失败（前 10 项已打印）",
        failures.len()
    );
}
