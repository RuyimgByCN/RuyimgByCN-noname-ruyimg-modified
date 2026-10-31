//! NC09/NC03 座位投影与产品会话面：**观察者/座位视图不泄漏隐藏信息**，
//! 错误可见，投影在存读档后保持一致。
//!
//! 口径（与 `native-extensions.md`、方案 §NC03/§NC09 一致）：
//! - 自己的手牌给出具体牌 id；他人手牌为 `Hidden`（仅数量）；
//! - 牌库仅数量；弃牌堆/桌面/装备/判定为公开；
//! - 开放选择：询问存在与类型公开，**选项内容仅观察者可见**；
//! - 非法命令返回**明确拒绝原因**（不是空结果）；
//! - 归档往返后，同一座位的投影逐字段一致。

use engine_native::view::{ChoiceView, HandView};
use engine_native::{Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack
}

fn drive_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..30 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<engine_native::ChoiceOption> =
                if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                    vec![engine_native::ChoiceOption::Pass]
                } else if c.spec.allow_cancel {
                    Vec::new()
                } else {
                    c.spec
                        .options
                        .iter()
                        .take(c.spec.min.max(1))
                        .copied()
                        .collect()
                };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            continue;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
}

/// 座位投影：自己的手牌可见、他人手牌仅数量、牌库仅数量。
#[test]
fn nc09_seat_projection_hides_others_cards() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261201, 4, &[&pack]).expect("建局");
    drive_to_play(&mut e, 0);

    let view = e.view_for(0);
    assert_eq!(view.seat, 0);
    let me = view.players.iter().find(|p| p.seat == 0).expect("自己");
    match &me.hand {
        HandView::Own(cards) => {
            assert_eq!(cards.len(), e.debug_hand(0).len(), "自己手牌逐张可见")
        }
        other => panic!("自己的手牌应为可见: {other:?}"),
    }
    for p in view.players.iter().filter(|p| p.seat != 0) {
        assert!(
            matches!(p.hand, HandView::Hidden),
            "他人手牌必须隐藏（座位 {}）: {:?}",
            p.seat,
            p.hand
        );
        assert_eq!(
            p.hand_count,
            e.debug_hand(p.seat).len(),
            "他人手牌仅数量（座位 {}）",
            p.seat
        );
    }
    // 牌库仅数量
    assert_eq!(view.deck_count, e.state_core().deck.len(), "牌库仅数量");
    // 弃牌堆公开
    assert_eq!(view.discard, e.state_core().discard.clone(), "弃牌堆公开");
    // 观战者（视角座位 = 无）：不含任何手牌 id
    let spectator = e.view_for(engine_native::ids::PlayerId::MAX);
    for p in &spectator.players {
        assert!(
            matches!(p.hand, HandView::Hidden),
            "观战者视图不得含手牌 id（座位 {}）",
            p.seat
        );
    }
}

/// 开放选择：询问公开、选项内容仅观察者可见。
#[test]
fn nc09_choice_options_are_observer_only() {
    let mut e = Engine::new(20261202, 4);
    // 推进到出现开放选择
    for _ in 0..40 {
        if e.open_choice().is_some() {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    let choice = e.open_choice().cloned().expect("应有开放选择");
    let observer = choice.observer;
    let own = e.choice_view_for(Some(observer)).expect("观察者可见询问");
    match own {
        ChoiceView::Own { options, .. } => assert_eq!(
            options.len(),
            choice.spec.options.len(),
            "观察者可见完整选项"
        ),
        other => panic!("观察者应见完整选项: {other:?}"),
    }
    let other_seat = (0..4u8).find(|p| *p != observer).expect("其他座位");
    match e.choice_view_for(Some(other_seat)) {
        Some(ChoiceView::Other { .. }) | None => {}
        other => panic!("非观察者不得见选项内容: {other:?}"),
    }
}

/// 错误可见：非法命令返回明确拒绝原因（不是空结果/静默成功）。
#[test]
fn nc09_illegal_commands_are_visibly_rejected() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261203, 4, &[&pack]).expect("建局");
    drive_to_play(&mut e, 0);

    // ① 非本人回合推进阶段（换到他人回合后由 0 号提交）
    let other = (1..4u8).find(|p| *p != e.turn().current).expect("其他座位");
    let out = e.submit(GameCommand {
        actor: other,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    match out {
        engine_native::CommandOutcome::Rejected(reason) => {
            assert!(
                matches!(reason, engine_native::RejectReason::NotYourTurn),
                "非本人回合应明确拒绝: {reason:?}"
            );
        }
        other => panic!("应被拒绝: {other:?}"),
    }

    // ② 不存在的牌
    let out2 = e.submit(GameCommand {
        actor: e.turn().current,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: 65000,
            targets: vec![],
        },
    });
    match out2 {
        engine_native::CommandOutcome::Rejected(reason) => {
            assert!(
                matches!(reason, engine_native::RejectReason::CardNotInHand),
                "未知牌应明确拒绝: {reason:?}"
            );
        }
        other => panic!("应被拒绝: {other:?}"),
    }

    // ③ 过期修订号
    let out3 = e.submit(GameCommand {
        actor: e.turn().current,
        expected_revision: e.revision().wrapping_sub(1),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    match out3 {
        engine_native::CommandOutcome::Rejected(reason) => assert!(
            matches!(reason, engine_native::RejectReason::StaleRevision),
            "过期修订号应明确拒绝: {reason:?}"
        ),
        other => panic!("应被拒绝: {other:?}"),
    }
}

/// 存读档后座位投影一致（跨进程恢复的可见面不漂移）。
#[test]
fn nc09_projection_survives_archive_roundtrip() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261204, 4, &[&pack]).expect("建局");
    drive_to_play(&mut e, 0);
    let before = format!("{:?}", e.view_for(0));
    let before_spec = format!("{:?}", e.view_for(engine_native::ids::PlayerId::MAX));

    let bytes = engine_native::archive::encode_archive(&e.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码");
    let restored = Engine::restore_with_extensions(&snapshot, &[&pack]).expect("带扩展恢复");
    assert_eq!(
        format!("{:?}", restored.view_for(0)),
        before,
        "座位投影在恢复后一致"
    );
    assert_eq!(
        format!("{:?}", restored.view_for(engine_native::ids::PlayerId::MAX)),
        before_spec,
        "观战投影在恢复后一致"
    );
}
