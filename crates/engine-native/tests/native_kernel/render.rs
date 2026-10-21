use super::*;

/// 渲染帧的国战将领行组合视图隔离：暗置将只出现在本人帧；
/// 明置将公开；观战者帧无任何暗置将。
#[test]
fn render_frame_guozhan_generals_isolation() {
    use engine_native::state::GameMode;
    let mut e = Engine::new(972, 3);
    let mut snap = e.save();
    snap.core.mode = GameMode::Kingdoms;
    snap.core.players[0].general_slots = [Some(2), None]; // 曹操 魏（暗置）
    snap.core.players[1].general_slots = [Some(17), None]; // 孙权 吴（暗置）
    snap.core.players[2].general_slots[0] = Some(5); // 关羽 蜀（暗置）
    e = Engine::restore(&snap).expect("restore");
    // 座位 0 明置主将
    advance_to_play(&mut e, 0);
    assert!(matches!(
        submit_cmd(&mut e, 0, GameAction::RevealGeneral { slot: 0 }),
        CommandOutcome::Accepted { .. }
    ));

    // 本人帧：自己的暗置/明置将都可见
    let f0 = e.render_frame(Some(0), 72);
    assert!(f0.contains("曹操"), "本人帧应含自己明置的主将");
    // 他人帧：座位 0 明置将可见；座位 1 本人暗置将可见（自己的）；他
    // 人的（座位 2 关羽）不可见
    let f1 = e.render_frame(Some(1), 72);
    assert!(f1.contains("曹操"), "明置将公开");
    assert!(f1.contains("孙权"), "本人帧含自己的暗置将");
    assert!(!f1.contains("关羽"), "他人暗置将不得入帧");
    // 观战者帧：只有明置将
    let fs = e.render_frame(None, 72);
    assert!(fs.contains("曹操") && !fs.contains("孙权") && !fs.contains("关羽"));
    // 座位 2 帧：无曹操？不——曹操已明置可见；无孙权（他人暗置）
    let f2 = e.render_frame(Some(2), 72);
    assert!(f2.contains("曹操") && !f2.contains("孙权") && f2.contains("关羽"));
}
