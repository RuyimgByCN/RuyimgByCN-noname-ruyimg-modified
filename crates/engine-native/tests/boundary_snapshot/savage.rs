use super::*;

/// N05 组合回归：南蛮多目标无懈链 → 乐判定翻牌触发真实 DeckReshuffled →
/// 首个无懈窗口挂起保存/恢复后继续，终态与基准线完全一致。
#[test]
fn savage_multi_nullify_judge_reshuffle_restore() {
    // 基准线
    let mut base = build_savage_nullify_reshuffle();
    drive_nullify_windows(&mut base);
    let hash_base = continue_to_judge_reshuffle(&mut base);

    // 快照线：首个无懈窗口挂起点保存/恢复，同脚本继续
    let b = build_savage_nullify_reshuffle();
    assert!(b.open_choice().is_some(), "应停在首个无懈窗口");
    let snap = b.save();
    let mut r = Engine::restore(&snap).expect("restore");
    assert_eq!(r.state_hash(), b.state_hash(), "恢复件与挂起点同哈希");
    drive_nullify_windows(&mut r);
    let hash_r = continue_to_judge_reshuffle(&mut r);
    assert_eq!(hash_r, hash_base, "恢复线终态必须与基准一致");
    assert_eq!(
        r.log()
            .iter()
            .filter(|l| matches!(l, LE::DeckReshuffled))
            .count(),
        1,
        "恢复线同样恰好发生一次真实洗牌"
    );
}
