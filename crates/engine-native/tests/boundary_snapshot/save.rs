use super::*;

// ---------------------------------------------------------------------------
// 挂起机器快照与恢复
// ---------------------------------------------------------------------------

/// 静止点保存 → 恢复 → 哈希一致；恢复后继续推进与未恢复线完全同哈希。
#[test]
fn save_restore_at_rest_continues_identically() {
    let mut e = Engine::new(11, 4);
    advance_to_play(&mut e, 0);
    let snap = e.save();
    let mut r = Engine::restore(&snap).expect("restore");
    assert_eq!(r.state_hash(), e.state_hash(), "恢复后哈希必须一致");
    let rev_before = e.revision();
    assert_eq!(r.revision(), rev_before);
    // 两线各推进两个阶段（含弃牌挂起），每步后哈希保持一致
    for _ in 0..2 {
        drive_phase(&mut e);
        drive_phase(&mut r);
        assert_eq!(r.state_hash(), e.state_hash(), "恢复后继续推进仍同哈希");
    }
}
