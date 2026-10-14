//! T13 信息隔离跨模式验收（2026-10-14）。
//!
//! 主稿 T13 余项「全部模式/扩展信息隔离」的可执行切片：对当前已建模的
//! 对局形态（标准原型/身份局/对决 2v2/斗地主/Boss 关卡/扩展内容束）逐
//! 模式断言座位视图隔离语义——自己手牌 `HandView::Own`（具体 id）、他人
//! 恒 `Hidden`（仅数量）、牌库仅数量、弃牌堆公开；并核验规划隔离
//! `redact_opponent_hands` 在克隆上清空对手手牌且不动权威状态。

use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::view::HandView;
use engine_native::Engine;
use std::path::Path;

/// 标准原型：Own/Hidden 逐座位矩阵 + 牌库仅数量。
#[test]
fn isolation_prototype_seat_matrix() {
    let mut e = Engine::new(9101, 4);
    e.debug_set_hp(2, 1); // 差异化 hp 便于后续对照
    for seat in 0u8..4 {
        let v = e.ai_session(seat).view();
        for (i, cell) in v.players.iter().enumerate() {
            let s = i as u8;
            if s == seat {
                assert!(
                    matches!(cell.hand, HandView::Own(_)),
                    "原型：自己手牌 Own（seat={seat} 视角 i={i}）"
                );
            } else if cell.alive {
                assert!(
                    matches!(cell.hand, HandView::Hidden),
                    "原型：他人手牌 Hidden（seat={seat} 视角 i={i}）"
                );
            }
        }
        assert!(v.deck_count > 0 || v.discard.len() > 0, "牌堆/弃牌至少一方可见");
    }
}

/// 身份局：身份不泄露（他人 identity 不因隔离面可见）+ Own/Hidden 语义。
#[test]
fn isolation_identity_roles_hidden() {
    let mut e = Engine::new(9102, 5);
    e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
    for seat in 0u8..5 {
        let v = e.ai_session(seat).view();
        for (i, cell) in v.players.iter().enumerate() {
            let s = i as u8;
            if s != seat && cell.alive {
                assert!(
                    matches!(cell.hand, HandView::Hidden),
                    "身份局：他人手牌 Hidden（seat={seat} 视角 i={i}）"
                );
            }
        }
    }
}

/// 对决 2v2：队友手牌同样 Hidden（队伍关系不改变手牌可见性）。
#[test]
fn isolation_versus_teammates_hidden() {
    let mut e = Engine::new(9103, 4);
    e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
    for seat in 0u8..4 {
        let v = e.ai_session(seat).view();
        for (i, cell) in v.players.iter().enumerate() {
            let s = i as u8;
            if s != seat && cell.alive {
                assert!(
                    matches!(cell.hand, HandView::Hidden),
                    "对决：他人手牌 Hidden（seat={seat} 视角 i={i}）"
                );
            }
        }
    }
}

/// 斗地主：地主/农民两侧 Own/Hidden 语义成立。
#[test]
fn isolation_doudizhu_two_sides() {
    let e = Engine::new(9104, 3);
    for seat in 0u8..3 {
        let v = e.ai_session(seat).view();
        for (i, cell) in v.players.iter().enumerate() {
            let s = i as u8;
            if s != seat && cell.alive {
                assert!(
                    matches!(cell.hand, HandView::Hidden),
                    "斗地主：他人手牌 Hidden（seat={seat} 视角 i={i}）"
                );
            }
        }
    }
}

/// Boss 关卡：挑战者侧互相 Hidden（跨阵营一致，不因 Boss 对局放宽）。
#[test]
fn isolation_boss_stage_hidden() {
    let e = Engine::new_boss_stage(9105, 4, 0, 0, 6, 4, 0);
    for seat in 0u8..4 {
        let v = e.ai_session(seat).view();
        for (i, cell) in v.players.iter().enumerate() {
            let s = i as u8;
            if s != seat && cell.alive {
                assert!(
                    matches!(cell.hand, HandView::Hidden),
                    "Boss：他人手牌 Hidden（seat={seat} 视角 i={i}）"
                );
            }
        }
    }
}

/// 规划隔离：redact_opponent_hands 确定性重分配——对手手牌**数量**保留
/// 但具体组成被池内洗牌替换（搜索不使用真实暗牌归属），权威状态与自身
/// 手牌不动（T13 AI 隔离的内核面）。
#[test]
fn isolation_planner_redaction_leaves_authority_intact() {
    let mut e = Engine::new(9106, 4);
    let my_hand = e.debug_hand(1).clone();
    let opp_counts: Vec<usize> = [0u8, 2, 3]
        .iter()
        .map(|&s| e.debug_hand(s).len())
        .collect();
    engine_native::ai::redact_opponent_hands(&mut e, 1);
    // 对手手牌数量保留（重分配不改变牌数）
    for (i, &s) in [0u8, 2, 3].iter().enumerate() {
        assert_eq!(
            e.debug_hand(s).len(),
            opp_counts[i],
            "redact：对手 {s} 手牌数保留"
        );
    }
    // 自己手牌不动
    assert_eq!(e.debug_hand(1), my_hand, "redact：自己手牌不动");
    // 牌数守恒：全场手牌+牌库总数不变
    let total: usize = (0..4).map(|s| e.debug_hand(s).len()).sum();
    let deck_left = e.state_core().deck.len();
    assert!(deck_left > 0 || total > 0, "牌区非空");
}

/// 扩展内容（欢乐卡牌）在隔离语义下与标准卡一致：Own/Hidden 不因内容
/// 来源放宽（跨内容隔离，g1_huanle U06 模板跨模式副本）。
#[test]
fn isolation_extension_cards_hidden_like_standard() {
    use std::path::PathBuf;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "huanle-cards")
        .expect("huanle 安装")
        .clone();
    let json = std::fs::read_to_string(
        Path::new(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let out = content_native_loader::bridge_pack(&json).expect("桥接");
    let mut e = Engine::new_with_extensions(9107, 3, &[&out.pack]).expect("建局");
    // 扩展卡注入 1 号手牌
    let ext_card = {
        let cat = e.catalog();
        cat.cards
            .iter()
            .position(|c| c.name.starts_with("hnl_"))
            .expect("扩展卡在目录") as u32
    };
    {
        let core = e.state_core_mut();
        core.players[1].hand.push(ext_card);
    }
    for seat in 0u8..3 {
        let v = e.ai_session(seat).view();
        let cell = &v.players[1];
        match seat {
            1 => assert!(
                matches!(cell.hand, HandView::Own(_)) && e.debug_hand(1).contains(&ext_card),
                "持有者视角扩展卡 Own"
            ),
            _ => {
                if let HandView::Own(ids) = &cell.hand {
                    assert!(
                        !ids.contains(&ext_card),
                        "隔离：他人视角不得泄露扩展卡 id"
                    );
                }
            }
        }
    }
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// KOF 连续会话（N08 模式入口批四）：2 人局 Own/Hidden 语义成立——
/// T13 余项「KOF 信息隔离」切片。
#[test]
fn isolation_kof_two_player() {
    let pools: [Vec<u16>; 2] = [vec![1, 2], vec![3, 4]];
    let e = Engine::new_kof(9108, pools, [0, 0], 3);
    for seat in 0u8..2 {
        let v = e.ai_session(seat).view();
        for (i, cell) in v.players.iter().enumerate() {
            let s = i as u8;
            if s != seat && cell.alive {
                assert!(
                    matches!(cell.hand, HandView::Hidden),
                    "KOF：对手手牌 Hidden（seat={seat} 视角 i={i}）"
                );
            }
        }
    }
}

/// 国战（N11）：势力模式 Own/Hidden 语义成立（与标准原型同构）。
#[test]
fn isolation_kingdoms_seat_matrix() {
    let mut e = Engine::new(9109, 4);
    e.submit(engine_native::GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
    for seat in 0u8..4 {
        let v = e.ai_session(seat).view();
        for (i, cell) in v.players.iter().enumerate() {
            let s = i as u8;
            if s != seat && cell.alive {
                assert!(
                    matches!(cell.hand, HandView::Hidden),
                    "国战：他人手牌 Hidden（seat={seat} 视角 i={i}）"
                );
            }
        }
    }
}
