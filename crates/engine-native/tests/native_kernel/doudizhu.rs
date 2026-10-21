use super::*;

/// 斗地主 binglin 变体开局（规则 v205）：地主 5 张/农民 4 张；
/// 标准变体各 4 张；开局后存档往返手牌一致。
#[test]
fn doudizhu_binglin_opening_draw() {
    // binglin：地主（0 号）5 张，农民各 4 张
    let mut e = Engine::new_doudizhu_binglin(2150, 3, 0);
    assert_eq!(e.debug_hand(0).len(), 5, "binglin 地主开局 5 张");
    assert_eq!(e.debug_hand(1).len(), 4, "binglin 农民开局 4 张");
    assert_eq!(e.debug_hand(2).len(), 4, "binglin 农民开局 4 张");
    // 模式身份不变：地主死亡 → 农民胜（胜负面与标准一致）
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 1);
    e.debug_clear_hand(0); // 清手牌防桃自救询问
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(0), "地主应死亡");
    // 开局后存档往返（换标准变体局验证）：
    let s2 = Engine::new_doudizhu_binglin(2151, 3, 1);
    let snap = s2.save();
    let r = Engine::restore(&snap).expect("binglin 局存档往返");
    assert_eq!(r.debug_hand(1).len(), 5, "恢复后地主（1 号）手牌一致");
    assert_eq!(r.debug_hand(0).len(), 4, "恢复后农民手牌一致");
    // 标准变体回归：各 4 张
    let s3 = Engine::new_doudizhu(2152, 3, 0);
    for p in 0..3 {
        assert_eq!(s3.debug_hand(p).len(), 4, "标准斗地主各 4 张");
    }
}

/// 斗地主（P2/N08 模式入口批二，规则 v81）：地主死亡 → 农民胜；
/// 农民全灭 → 地主胜；无击杀奖惩；存档往返。
#[test]
fn doudizhu_mode_victory_and_roundtrip() {
    let mut e = Engine::new_doudizhu(2101, 3, 0);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // ① 农民 2 死于地主之手（无奖惩：地主手牌数不变）；未终局（1 号存活）
    e.debug_set_hp(2, 1);
    let hand0 = e.debug_hand(0).len();
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(2));
    assert_eq!(e.debug_hand(0).len(), hand0, "斗地主无击杀奖惩");
    assert!(e.outcome().is_none(), "1 号存活，未终局");

    // ② 存档往返：模式表一致
    {
        use engine_native::archive::{decode_archive, encode_archive};
        let bytes = encode_archive(&e.save());
        let back = decode_archive(&bytes).expect("解码");
        assert_eq!(back.core.mode, e.save().core.mode, "斗地主模式往返一致");
    }

    // ③ 地主（0）死于农民之手 → 农民胜
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(0));
    assert!(
        matches!(e.outcome(), Some(o) if o.winner == Winner::Doudizhu { lord_won: false }),
        "{:?}",
        e.outcome()
    );

    // ④ 反向：农民全灭 → 地主胜（新局）
    let mut e = Engine::new_doudizhu(2103, 3, 0);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    for seat in [1u8, 2] {
        e.debug_clear_hand(0); // 防奖惩摸牌含桃干扰（与身份局测试同口径）
        e.debug_set_hp(seat, 1);
        let out = e.debug_deal_damage(Some(0), seat, 1, DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
        if seat == 1 {
            assert!(e.outcome().is_none(), "2 号农民存活，未终局");
        }
    }
    assert!(
        matches!(e.outcome(), Some(o) if o.winner == Winner::Doudizhu { lord_won: true }),
        "{:?}",
        e.outcome()
    );
}
