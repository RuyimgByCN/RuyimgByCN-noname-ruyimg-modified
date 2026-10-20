//! NC08：**死键抽查** —— 可达性扫描里「无任何声明路径」的宿主键，是否真的进不了目录。
//!
//! 背景：`scripts/host_key_reachability.py` 扫出 **94 个键**既不在包数据、也不在
//! `content/host.rs` 的 `HostSkillDecl` 表里。而**运行时授予**（`grant_*`）都要经
//! `catalog.skill_by_name(...)` ⇒ **不在目录里的键无法被授予**。
//!
//! 本用例装载**全部 41 个包**，抽查其中若干键：**断言它们不在 `catalog.skills` 里**。
//! 若通过 ⇒ 「死键」结论有实测支撑（不是只靠静态推理）。

use std::path::PathBuf;

const SAMPLE: [&str; 6] = [
    "host:baimang2",
    "host:baimang3x",
    "host:chiyan2",
    "host:qingmu2",
    "host:xuanlin2",
    "host:zhaomingdan",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

#[test]
fn dead_host_keys_are_absent_from_catalog() {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root())
        .expect("read packages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    // 全部包同时装载（同名包按路径顺序；此处只关心 catalog 里的技能名集合）
    let mut packs: Vec<engine_native::content::extension::ExtensionPack> = Vec::new();
    for d in &dirs {
        let assets = d.join("assets");
        for f in ["cards.json", "character.json"] {
            let p = assets.join(f);
            if !p.is_file() {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            let Ok(br) = content_native_loader::bridge_pack(&text) else { continue };
            if !br.pack.host_skills.is_empty() || !br.pack.vm_skills.is_empty() {
                packs.push(br.pack);
            }
        }
    }
    let refs: Vec<&engine_native::content::extension::ExtensionPack> = packs.iter().collect();
    let engine = engine_native::Engine::new_with_extensions(91, 4, &refs).expect("建局");

    let mut present = Vec::new();
    for k in SAMPLE {
        let short = k.trim_start_matches("host:");
        let hit = engine
            .catalog()
            .skills
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(short) || s.name.eq_ignore_ascii_case(k));
        if hit {
            present.push(k);
        }
    }
    eprintln!(
        "DBG 死键抽查：装载 {} 个包；样本 {} 个；出现在目录里的 {} 个 {:?}",
        packs.len(),
        SAMPLE.len(),
        present.len(),
        present
    );
    assert!(
        present.is_empty(),
        "样本键不应出现在目录里（否则说明还有别的挂载路径）：{present:?}"
    );
}
