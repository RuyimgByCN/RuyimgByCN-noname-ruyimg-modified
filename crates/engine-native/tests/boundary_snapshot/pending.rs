use super::*;

/// 挂起点恢复件的摘要与未恢复件一致（含机器快照字节）。
#[test]
fn pending_state_hash_stable_across_save_restore() {
    let e = setup_sancai_pending();
    let hash_before = e.state_hash();
    let snap = e.save();
    let r = Engine::restore(&snap).unwrap();
    assert_eq!(r.state_hash(), hash_before);
}
