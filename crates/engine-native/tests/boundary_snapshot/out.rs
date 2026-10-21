use super::*;

/// actor 越界 → UnknownActor，不 panic。
#[test]
fn out_of_bounds_actor_rejected() {
    let (mut e, _) = setup_play_with_sha();
    let out = submit_cmd(&mut e, 250, GameAction::NextPhase);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::UnknownActor)),
        "{out:?}"
    );
}

/// 越界目标/牌 ID 显式拒绝（此前路径可 panic）。
#[test]
fn out_of_bounds_ids_rejected_not_panic() {
    let (mut e, sha) = setup_play_with_sha();
    let before = e.state_hash();
    // 杀 → 越界目标
    let out = play(&mut e, sha, vec![200]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "{out:?}"
    );
    assert_eq!(e.state_hash(), before);
    // 铁索 → 越界目标（debug_place 本身改状态，以其之后为新基准）
    let chain = e.debug_place(0, "chain");
    let before_chain = e.state_hash();
    let out = play(&mut e, chain, vec![200]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "{out:?}"
    );
    // 越界牌 → CardNotInHand
    let out = play(&mut e, u32::MAX, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::CardNotInHand)),
        "{out:?}"
    );
    assert_eq!(e.state_hash(), before_chain);
}
