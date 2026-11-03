//! R05-G（存档 107）：`detached_cards` 实体区入档 + golden 哈希域补 detached。
//!
//! 三面验收（先红后绿）：
//! 1. 守恒——真实双枯竭路径产出非空 detached，save→encode→decode 往返
//!    逐条相等，且 restore 路径牌归属校验（N04-f）通过；
//!    修前红＝解码侧 `Vec::new()` 默认吞掉整区。
//! 2. 哈希域——两个仅 detached 内容不同的状态摘要不同；
//!    修前红＝摘要不含 detached（漂移源）。
//! 3. 版本门禁——升号前真实 106 挂起档（探针产出，含非空 detached 的
//!    save 状态）在 107 下按 `UnsupportedFormatVersion(106)` 拒收，
//!    无迁移（对齐 R05 合同：v106 及更早按版本门禁拒绝）。

use engine_native::archive::{decode_archive, encode_archive, ArchiveError};
use engine_native::event::DetachedKind;
use engine_native::{CommandOutcome, Engine};

/// 双枯竭布阵（牌守恒：不销毁任何牌）：弃堆清空、牌堆其余全部移入
/// seat0 手牌，仅留堆顶 1 张——`debug_start_draw_phase_fixed` 的真实 D
/// 帧请求 2 张：R 抽走堆顶后 deck/discard 双空（kernel/draw.rs §4），
/// held 逐张转 detached（Unclaimed）。
fn exhausted_engine(seed: u64) -> Engine {
    let mut e = Engine::new(seed, 3);
    {
        let core = e.state_core_mut();
        let keep = core.deck.pop().expect("开局牌堆非空");
        let rest: Vec<u32> = core.deck.drain(..).chain(core.discard.drain(..)).collect();
        core.players[0].hand.extend(rest);
        core.deck.push(keep);
    }
    let outcome = e.debug_start_draw_phase_fixed(0, 2);
    assert!(
        !matches!(outcome, CommandOutcome::Rejected(_)),
        "固定张数 D 帧不得被拒: {outcome:?}"
    );
    assert!(
        !e.state_core().detached_cards.is_empty(),
        "双枯竭必须产出非空 detached（真实生产者 kernel/draw.rs Exhausted）"
    );
    e
}

/// 红1（守恒）：detached 随存档 107 往返逐条守恒，restore 归属校验通过。
#[test]
fn r05g_detached_zone_survives_archive_round_trip() {
    let e = exhausted_engine(65333);
    let snapshot = e.save();
    let expected = snapshot.core.detached_cards.clone();
    assert!(
        expected
            .iter()
            .all(|(_, kind)| matches!(kind, DetachedKind::Unclaimed)),
        "双枯竭产出 Unclaimed：{expected:?}"
    );
    let decoded = decode_archive(&encode_archive(&snapshot)).expect("107 编码可解");
    assert_eq!(
        decoded.core.detached_cards, expected,
        "修前红：解码侧 detached 为 Vec::new() 默认（整区丢失）"
    );
    // restore 路径：N04-f 牌归属多重集校验把 detached 计为独立牌区。
    let restored = Engine::restore(&decoded).expect("含 detached 的快照可恢复");
    assert_eq!(restored.state_core().detached_cards, expected);
}

/// 红2（哈希域）：仅 detached 内容不同的状态摘要不同——
/// kind 维度（Unclaimed vs Destroyed）与存在维度（有 vs 无）各一面。
#[test]
fn r05g_detached_is_in_hash_domain() {
    let mut unclaimed = Engine::new(65341, 3);
    let mut destroyed = Engine::new(65341, 3);
    let mut absent = Engine::new(65341, 3);
    let card = {
        let core = unclaimed.state_core_mut();
        core.deck.pop().expect("开局牌堆非空")
    };
    for e in [&mut destroyed, &mut absent] {
        let popped = e.state_core_mut().deck.pop().expect("开局牌堆非空");
        assert_eq!(popped, card, "同种子同布阵：三状态仅 detached 不同");
    }
    unclaimed
        .state_core_mut()
        .detached_cards
        .push((card, DetachedKind::Unclaimed));
    destroyed
        .state_core_mut()
        .detached_cards
        .push((card, DetachedKind::Destroyed));
    assert_ne!(
        unclaimed.state_hash(),
        destroyed.state_hash(),
        "修前红：哈希域不含 detached——kind 维度不可区分"
    );
    assert_ne!(
        unclaimed.state_hash(),
        absent.state_hash(),
        "修前红：哈希域不含 detached——存在维度不可区分"
    );
}

/// 门禁（版本拒收）：升号前探针产出的真实 106 挂起档（save 状态含
/// 非空 detached，但 106 布局不编码该区）在 107 下拒收，原件不改动、
/// 无迁移（参照 gaiming-window-331.ngsa 先例）。
#[test]
fn r05g_genuine_106_archive_with_detached_is_version_rejected() {
    let bytes = include_bytes!("fixtures/detached-106.ngsa");
    assert_eq!(&bytes[0..4], b"NGSA");
    assert_eq!(
        u16::from_le_bytes([bytes[4], bytes[5]]),
        106,
        "fixture 必须是升号前的真实 106 字节"
    );
    match decode_archive(bytes) {
        Err(ArchiveError::UnsupportedFormatVersion(106)) => {}
        other => panic!("期望 107 门禁拒收 106，得到 {other:?}"),
    }
}
