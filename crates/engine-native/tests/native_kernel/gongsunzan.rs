use super::*;

/// P2/N08 标准缺失将第三批（2026-10-07，6/8）：公孙瓒「义从」（refresh
/// 改版口径，上游唯一 reyicong 定义）——锁定距离修正：自己算与他人的
/// 距离 -1（无条件）；他人算与自己的距离 +1（自己体力值 ≤2 时）。
#[test]
fn gongsunzan_assembly_yicong_distance() {
    let mut e = Engine::new(90, 4);
    advance_to_play(&mut e, 0);
    e.debug_assemble_general(1, "公孙瓒");
    assert!(e.debug_has_skill(1, "义从"), "装配公孙瓒授予义从");
    assert_eq!(e.save().core.players[1].max_hp, 4, "公孙瓒体力上限 4");

    // 三人局中 0↔1、1↔2 基础座位距离均为 1。
    // ① 义从持有者体力 4（>2）：1→0 与 1→2 距离 -1 → 饱和下限仍为 1；
    //    0→1 无 +1（持有者体力 >2）→ 仍 1。
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 1, 0),
        1,
        "义从持有者算向他人（饱和下限）"
    );
    assert_eq!(e.state_core().computed_distance(e.catalog(), 1, 2), 1);
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        1,
        "持有者高体力时他人算向他不加成"
    );

    // ② 持有者掉到 2 体力（≤2）：他人算向 1 号 +1 → 0→1 与 2→1 均为 2；
    //    1→0 仍 -1 → 1。
    e.debug_set_hp(1, 2);
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        2,
        "持有者低体力：他人算向他 +1"
    );
    assert_eq!(e.state_core().computed_distance(e.catalog(), 2, 1), 2);
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 1, 0),
        1,
        "持有者外向 -1 不受体力影响"
    );

    // ③ 与 +1 马（防御马）叠加：0 号装 +1 马（影响他人→0 的距离）：
    //    1→0 = 1(座) +1(0号防御马) -1(义从外向) = 1（抵消）；
    //    0→1 = 1(座) +1(义从Incoming，持有者 hp≤2) = 2（0 号的马不影响 0→1）。
    let horse = e.debug_place(0, "horse_plus");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: horse,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备+1马: {out:?}"
    );
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 1, 0),
        1,
        "持有者外向 -1 与对方防御马抵消"
    );
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        2,
        "义从 incoming 与对方防御马无交互"
    );
}
