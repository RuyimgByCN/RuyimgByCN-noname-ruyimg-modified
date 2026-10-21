use super::*;

/// 阴间系收尾片（规则 v255）：爆敛（End 摸 2）+强征（End 每名其他有手
/// 牌者首张手牌收归）+醉酒（sha 伤害 +1）+枭首（End 首个 hp≥self 其他
/// 存活受 3）。白无常/牛头条目；juece 界绝策[refresh 包]/蛮甲[extra-equip
/// 藤甲]/niepan[boss 本地]混入未落登记；黑无常/马面随其技批。
#[test]
fn boss_yinjian_baolian_qiangzheng_zuijiu_xiaoshou() {
    // ① 白无常：爆敛 End 摸 2 + 强征收 0/2/3 各首张手牌 + 醉酒杀 +1
    let mut e = Engine::new_boss_stage(75001, 4, 1, 3, 6, 4, 0);
    let bw_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 32) as u16; // BOSS_GENERALS[32] 白无常
    let nt_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 36) as u16; // BOSS_GENERALS[36] 牛头
    e.debug_set_general_def(1, bw_def);
    e.debug_set_hp(1, 8);
    assert_eq!(
        e.catalog().general_meta(bw_def).map(|g| g.0),
        Some("白无常"),
        "白无常入目录"
    );
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_set_hp(0, 8);
    e.debug_set_hp(2, 8);
    e.debug_set_hp(3, 8);
    let _sha1 = e.debug_place(1, "sha");
    let _h0 = e.debug_place(0, "tao");
    let _h2 = e.debug_place(2, "shan");
    let _h3 = e.debug_place(3, "wugu");
    // 杀 0 号（醉酒 +1 → 2 伤害）
    play(&mut e, _sha1, vec![0]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), 6, "醉酒：杀伤害 +1（8-2）");
    // 结束阶段：爆敛摸 2 + 强征收 h0/h2/h3 首张
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    assert_eq!(
        e.debug_hand(1).len(),
        5,
        "爆敛摸 2 + 强征收 3（0/2/3 各首张）"
    );
    assert_eq!(e.debug_hand(0).len(), 0, "强征：0 号手牌被收");
    assert_eq!(e.debug_hand(2).len(), 0, "强征：2 号手牌被收");
    assert_eq!(e.debug_hand(3).len(), 0, "强征：3 号手牌被收");
    assert_eq!(e.debug_hp(1), 8, "醉酒只加伤害不加自伤");
    assert_zone_conservation(&e);

    // ② 枭首（牛头）：End 对首个 hp≥self 其他存活受 3
    let mut e2 = Engine::new_boss_stage(75002, 4, 1, 3, 6, 4, 0);
    e2.debug_set_general_def(1, nt_def);
    e2.debug_set_hp(1, 7);
    assert_eq!(
        e2.catalog().general_meta(nt_def).map(|g| g.0),
        Some("牛头"),
        "牛头入目录"
    );
    // 0/2/3 均满血 4 < 7：无 hp≥self 目标 → 不触发
    advance_to_play(&mut e2, 1);
    let h2_before = e2.debug_hp(2);
    next_phase(&mut e2); // Play→Discard（爆敛摸 2）
    drain_choices(&mut e2);
    // End 臂后无枭首伤害（无敌方 hp≥7）
    assert_eq!(e2.debug_hp(2), h2_before, "无 hp≥self 目标：枭首不触发");
    // 压 2 号至 7 → 触发枭首受 3
    e2.debug_set_hp(2, 7);
    next_phase(&mut e2); // Discard→End
    drain_choices(&mut e2);
    next_phase(&mut e2); // End→0 Judge
    drain_choices(&mut e2);
    // 推回 1 号 End：此时 2 号 hp7 ≥ 1 号 hp7 → 枭首 3 伤
    let _ = h2_before;
    assert_eq!(e2.debug_hp(2), 7, "前置：2 号 hp7");

    // ⑤ 枭首触发面扩展：压 2 号 hp 至 7 后（上方已置），驱动至 1 号
    //    End → 座位序首个 hp≥self 其他存活=2 号 → 3 点伤害（7→4）
    for _ in 0..24 {
        if e2.turn().current == 1 && e2.turn().phase == Phase::End {
            break;
        }
        if e2.outcome().is_some() {
            break;
        }
        if let Some(c) = e2.open_choice().cloned() {
            if c.spec.prompt == "dying_save" {
                answer(&mut e2, ChoiceOption::Pass);
                drain_choices(&mut e2);
            } else {
                answer_cards(&mut e2, &c);
            }
            drain_choices(&mut e2);
            continue;
        }
        match next_phase(&mut e2) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Suspended => {}
            CommandOutcome::Pending { choice } => {
                let is_dying = choice.spec.prompt == "dying_save";
                if is_dying {
                    answer(&mut e2, ChoiceOption::Pass);
                    drain_choices(&mut e2);
                } else {
                    answer_cards(&mut e2, &choice);
                }
            }
            CommandOutcome::Rejected(r) => panic!("np rejected {r:?}"),
        }
    }
    assert_eq!(e2.debug_hp(2), 4, "枭首：2 号受 3 伤害（7→4）");
    assert_eq!(e.debug_hp(0), 6, "前局不变（回归保护）");
    let snap = e2.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e2);
}
