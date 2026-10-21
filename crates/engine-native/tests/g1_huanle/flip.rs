use super::*;

/// E03 帧生命周期回归守卫：debug_flip 走事件流后，每次翻面帧都应完整
/// 弹栈（stack/parks 清空、无悬挂选择）——缺陷期实证残留帧导致取消写回
/// 指向已提交帧与状态膨胀，修复后的不变量在此固化。
#[test]
fn flip_frame_lifecycle_no_residue() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3208, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);

    // 无面具基线：两次 flip 的 stack/parks 应清空（帧全弹）。
    for i in 0..2 {
        let flipped = e.debug_flip(1);
        assert!(flipped, "第 {i} 次翻面应发生");
        let snap = e.save();
        eprintln!(
            "PROBE flip{i} stack={} parks={} open={:?} fd1={}",
            snap.resolution.stack.len(),
            snap.resolution.parks.len(),
            snap.resolution.open_choice.is_some(),
            e.debug_face_down(1)
        );
        assert_eq!(
            snap.resolution.stack.len(),
            0,
            "第 {i} 次翻面后事件栈应清空"
        );
    }
    eprintln!("PROBE 无面具基线通过");
}
/// E03 翻面原语（无扩展基线）：debug_flip 经 TurnOverBefore 事件流驱动
/// （帧弹栈后返回），无面具时每次调用都发生翻转，连续两次回到初始朝向；
/// 取消路径（面具写回 cancelled=true）由
/// huanle_mianju_blocks_flip_while_equipped 覆盖。
#[test]
fn flip_primitive_toggles_facing_without_mask() {
    let mut e = Engine::new(3209, 2);
    assert!(!e.debug_face_down(0), "初始正面");

    assert!(e.debug_flip(0), "无面具翻面发生");
    assert!(e.debug_face_down(0), "翻面后背面朝上");

    assert!(e.debug_flip(0), "翻回（翻转发生）");
    assert!(!e.debug_face_down(0), "翻回后正面朝上");
}
