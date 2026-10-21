use super::*;

// ---------------------------------------------------------------------------
// 基础链路
// ---------------------------------------------------------------------------

/// 目录经 Arc 共享：命令前后指针不变（状态克隆不复制目录）。
#[test]
fn catalog_shared_across_commands() {
    let mut e = Engine::new(7, 4);
    let before = e.catalog_ptr() as usize;
    e.debug_grant(1, "碎盾"); // 强制防止，避免任何选择挂起
    let sha = e.debug_place(0, "sha");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    match play(&mut e, sha, vec![1]) {
        CommandOutcome::Accepted { .. } => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(hp(&e, 1), 4);
    let after = e.catalog_ptr() as usize;
    assert_eq!(before, after, "catalog must not move across commands");
}
