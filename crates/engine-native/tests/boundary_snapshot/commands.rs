use super::*;

/// 非当前回合玩家的 PlayCard/NextPhase → NotYourTurn。
#[test]
fn commands_by_non_current_player_rejected() {
    let (mut e, sha) = setup_play_with_sha();
    assert_eq!(e.turn().current, 0);
    let before = e.state_hash();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NotYourTurn)),
        "{out:?}"
    );
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NotYourTurn)),
        "{out:?}"
    );
    assert_eq!(e.state_hash(), before);
}
